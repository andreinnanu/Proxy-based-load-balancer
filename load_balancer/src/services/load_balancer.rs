use std::{
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{Method, Request, Response, body::Incoming};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::TokioExecutor,
};
use tokio::{
    sync::RwLock,
    time::{self, timeout},
};
use tower::Service;

use crate::{
    services::{LoadBalancerState, Strategy},
    utils::{
        algorithm::{
            ALGORITHM_SWITCH_TIMEOUT_SEC, STRATEGY_QUERY_PARAM, SWITCH_ALGORITHM_ENDPOINT,
        },
        constants::health_check::{CHECK_INTERVAL_SEC, TIMEOUT_MS},
    },
};

struct ConnectionGuard {
    host: SocketAddr,
    state: Arc<RwLock<LoadBalancerState>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let state = self.state.clone();
        let host = self.host;
        
        tokio::spawn(async move {
            state.write().await.on_disconnect(&host);
        });
    }
}

#[derive(Clone)]
pub struct LoadBalancer {
    state: Arc<RwLock<LoadBalancerState>>,
    client: Arc<Client<HttpConnector, Incoming>>,
}

impl LoadBalancer {
    #[tracing::instrument(name = "Initializing load balancer", skip_all, level = "info")]
    pub fn new(state: Arc<RwLock<LoadBalancerState>>) -> Self {
        Self {
            state,
            client: Arc::new(Client::builder(TokioExecutor::new()).build(HttpConnector::new())),
        }
    }

    #[tracing::instrument(name = "Spawning adaptive engine", skip(self), level = "info")]
    pub async fn spawn_adaptive_engine(&self) {
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(CHECK_INTERVAL_SEC));

            loop {
                interval.tick().await;

                let next_strategy = match state.read().await.is_worker_overloaded() {
                    true => Strategy::LeastConnections,
                    false => Strategy::RoundRobin,
                };
                if state.write().await.set_algorithm(next_strategy) {
                    time::sleep(Duration::from_secs(ALGORITHM_SWITCH_TIMEOUT_SEC)).await;
                }
            }
        });
    }

    #[tracing::instrument(name = "Spawning health check task", skip(self), level = "info")]
    pub async fn spawn_health_check_task(&self) {
        let hosts: Vec<SocketAddr> = self.state.read().await.hosts.keys().cloned().collect();
        let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build_http();

        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(CHECK_INTERVAL_SEC));

            loop {
                interval.tick().await;
                for host in &hosts {
                    let req: Request<Full<Bytes>> = Request::builder()
                        .method(Method::GET)
                        .uri(format!("http://{host}/health"))
                        .body(Full::from(""))
                        .expect("Health check request builder");

                    match timeout(Duration::from_millis(TIMEOUT_MS), client.request(req)).await {
                        Ok(Ok(_res)) => {
                            state.write().await.set_host_health(host, true);
                        }
                        _ => {
                            state.write().await.set_host_health(host, false);
                        }
                    }
                }
            }
        });
    }
}

type ProxyResult =
    Result<Response<BoxBody<Bytes, hyper::Error>>, hyper_util::client::legacy::Error>;

impl Service<Request<Incoming>> for LoadBalancer {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper_util::client::legacy::Error;
    type Future = Pin<Box<dyn Future<Output = ProxyResult> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let state = self.state.clone();
        let client = self.client.clone();

        Box::pin(async move {
            match (req.method(), req.uri().path()) {
                (&Method::GET, SWITCH_ALGORITHM_ENDPOINT) => switch_algorithm(req, state).await,
                _ =>  {
                    let host = match state.write().await.get_host() {
                        Some(host) => host,
                        None => return Ok(build_http_response(503, "No available workers")),
                    };

                    let _guard = ConnectionGuard {
                        host,
                        state
                    };

                    forward_request(req, client, host).await
                },
            }
        })
    }
}

#[tracing::instrument(name = "Switch algorithm request", skip_all, level = "debug")]
async fn switch_algorithm(
    req: Request<Incoming>,
    state: Arc<RwLock<LoadBalancerState>>,
) -> ProxyResult {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    let strategy = match params
        .get(STRATEGY_QUERY_PARAM)
        .and_then(|s| s.parse::<Strategy>().ok())
    {
        Some(s) => s,
        None => return Ok(build_http_response(400, "Invalid or missing strategy")),
    };

    state.write().await.set_algorithm(strategy);
    Ok(build_http_response(200, "Strategy updated"))
}

#[tracing::instrument(name = "Forward request to worker", skip(req, client), level = "debug")]
async fn forward_request(
    req: Request<Incoming>,
    client: Arc<Client<HttpConnector, Incoming>>,
    host: SocketAddr
) -> ProxyResult {
    let mut parts = req.uri().clone().into_parts();

    if parts.scheme.is_none() {
        parts.scheme = Some(http::uri::Scheme::HTTP);
    }

    parts.authority = Some(host.to_string().parse().expect("Valid host expected"));

    let new_uri = match http::Uri::from_parts(parts) {
        Ok(uri) => uri,
        Err(_) => {
            return Ok(build_http_response(500, "Invalid URI"));
        }
    };

    let mut new_req = req.map(|b| b);
    *new_req.uri_mut() = new_uri;

    let resp = match client.request(new_req).await {
        Ok(r) => r,
        Err(_) => {
            return Ok(build_http_response(502, "Upstream request failed"));
        }
    };

    Ok(resp.map(|b| b.boxed()))
}

fn build_http_response(response_status: u16, msg: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    let body: BoxBody<Bytes, hyper::Error> = Full::new(Bytes::from(msg.to_owned()))
        .map_err(|e| match e {})
        .boxed();

    Response::builder()
        .status(response_status)
        .body(body)
        .expect("building a simple HTTP response should not fail")
}

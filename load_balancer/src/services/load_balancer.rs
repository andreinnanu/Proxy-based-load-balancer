use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{Method, Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, sync::RwLock};
use tower::Service;

use crate::{
    services::{LoadBalancerState, Strategy},
    utils::algorithm::{STRATEGY_QUERY_PARAM, SWITCH_ALGORITHM_ENDPOINT},
};

type ClientBuilder = hyper::client::conn::http1::Builder;

#[derive(Clone)]
pub struct LoadBalancer {
    state: Arc<RwLock<LoadBalancerState>>,
}

impl LoadBalancer {
    pub fn new(state: Arc<RwLock<LoadBalancerState>>) -> Self {
        Self { state }
    }
}

type ProxyResult = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;

impl Service<Request<Incoming>> for LoadBalancer {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = ProxyResult> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let state = self.state.clone();

        match (req.method(), req.uri().path()) {
            (&Method::GET, SWITCH_ALGORITHM_ENDPOINT) => {
                Box::pin(async move { switch_algorithm(req, state).await })
            }
            _ => Box::pin(async move {
                let host;
                {
                    host = state.write().await.get_host();
                }

                println!("Forwarding request to {host}");

                let stream = TcpStream::connect(host).await.unwrap();
                let io = TokioIo::new(stream);

                let (mut sender, conn) = ClientBuilder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .handshake(io)
                    .await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        println!("Connection failed: {err:?}");
                    }
                    state.write().await.on_disconnect(&host);
                });

                let resp = sender.send_request(req).await?;
                Ok(resp.map(|b| b.boxed()))
            }),
        }
    }
}

async fn switch_algorithm(
    req: Request<Incoming>,
    state: Arc<RwLock<LoadBalancerState>>,
) -> ProxyResult {
    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    match params
        .get(STRATEGY_QUERY_PARAM)
        .and_then(|s| s.parse::<Strategy>().ok())
    {
        Some(strategy) => {
            state.write().await.set_algorithm(strategy);
            Ok(build_http_response(200, ""))
        }
        _ => Ok(build_http_response(400, "")),
    }
}

fn build_http_response(response_status: u16, msg: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    let body: BoxBody<Bytes, hyper::Error> = Full::new(Bytes::from(msg.to_owned()))
        .map_err(|e| match e {})
        .boxed();

    Response::builder()
        .status(response_status)
        .body(body)
        .unwrap()
}

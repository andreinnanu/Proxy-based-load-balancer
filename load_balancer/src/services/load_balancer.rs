use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tower::Service;

type ClientBuilder = hyper::client::conn::http1::Builder;

#[derive(Clone, Debug)]
pub struct LoadBalancer {
    // TODO: update to actual shared state
    // the load balancing logic will be added here
    state: i32,
}

impl LoadBalancer {
    pub fn new(state: i32) -> Self {
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
        Box::pin(async move {
            // hardcoded worker
            let stream = TcpStream::connect(("0.0.0.0", 8081)).await.unwrap();
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
            });

            let resp = sender.send_request(req).await?;
            Ok(resp.map(|b| b.boxed()))
        })
    }
}

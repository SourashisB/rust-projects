use std::net::SocketAddr;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server as HyperServer;
use crate::router::Router;
use crate::handler::handle_request;

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new(addr: String) -> Self {
        let socket_addr: SocketAddr = addr.parse().expect("Invalid address");
        Server { addr: socket_addr }
    }

    pub async fn run(&self) -> Result<(), hyper::Error> {
        let router = Arc::new(Router::new());

        let make_svc = make_service_fn(move |_conn: &AddrStream| {
            let router = Arc::clone(&router);
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_request(req, Arc::clone(&router))
                }))
            }
        });

        let server = HyperServer::bind(&self.addr).serve(make_svc);
        tracing::info!("Server running on http://{}", self.addr);

        server.await
    }
}
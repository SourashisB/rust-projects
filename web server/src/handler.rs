use std::sync::Arc;
use hyper::{Body, Request, Response, StatusCode};
use crate::router::Router;

pub async fn handle_request(req: Request<Body>, router: Arc<Router>) -> Result<Response<Body>, hyper::Error> {
    let path = req.uri().path();

    if let Some(file_path) = router.get_route(path) {
        serve_file(file_path).await
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap())
    }
}

async fn serve_file(path: &str) -> Result<Response<Body>, hyper::Error> {
    match tokio::fs::read(path).await {
        Ok(contents) => Ok(Response::new(Body::from(contents))),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap()),
    }
}
use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, header};
use serde_json::json;

use crate::runtime::serverless::core::{AppPath, AppState};

pub(super) async fn service_handler(
    state: Arc<AppState>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let matched = state.router.at(req.uri().path());
    if let Ok(mtch) = matched {
        match mtch.value {
            AppPath::Api => {
                let operation = mtch.params.get("operation");
                let body = serde_json::to_vec(&json!({ "operation": operation })).unwrap();

                return Ok(Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Full::new(Bytes::from(body)))
                    .unwrap());
            }

            AppPath::Worker => {
                let segments = mtch.params.get("segments");
                let body = serde_json::to_vec(&json!({ "segments": segments })).unwrap();
                return Ok(Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Full::new(Bytes::from(body)))
                    .unwrap());
            }
        }
    }

    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

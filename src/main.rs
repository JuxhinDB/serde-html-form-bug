use axum::{routing::get, Router};
use axum_extra::extract::{Query, QueryRejection};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app().into_make_service())
        .await
        .unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/buggy", get(buggy_handler))
}

async fn handler(params: Result<Query<Params>, QueryRejection>) -> String {
    let params = params.unwrap_or_default();
    println!("filters: {:?}", params.filters);
    params.0.filters.join(",")
}

async fn buggy_handler(params: Result<Query<BuggyParams>, QueryRejection>) -> String {
    let params = params.unwrap_or_default();
    println!("filters: {:?}", params);
    params.0.parent.children.join(",")
}

#[derive(Debug, Deserialize)]
pub(crate) struct Params {
    #[serde(default, rename = "filter")]
    pub(crate) filters: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BuggyParams {
    #[serde(flatten)]
    pub(crate) parent: Filter,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Filter {
    #[serde(default, rename = "filter")]
    pub(crate) children: Vec<String>,
}

impl Default for Params {
    fn default() -> Self {
        Self { filters: vec![] }
    }
}

impl Default for BuggyParams {
    fn default() -> Self {
        Self {
            parent: Filter { children: vec![] },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_working() {
        assert_eq!(
            send_request_get_body("/", "filter=1&filter=2").await,
            r#"1,2"#,
        );

        assert_eq!(send_request_get_body("/", "filter=1").await, r#"1"#,);
    }

    #[tokio::test]
    async fn test_buggy() {
        assert_eq!(
            send_request_get_body("/buggy", "filter=1&filter=2").await,
            r#"1,2"#,
        );

        assert_eq!(send_request_get_body("/buggy", "filter=1").await, r#"1"#,);
    }

    async fn send_request_get_body(path: &str, query: &str) -> String {
        let body = app()
            .oneshot(
                Request::builder()
                    .uri(format!("{}?{}", path, query))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .into_body();
        let bytes = hyper::body::to_bytes(body).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

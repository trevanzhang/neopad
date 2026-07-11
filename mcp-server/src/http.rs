use crate::{auth::Auth, protocol::Server};
use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use neopad_core::Workspace;
use serde_json::Value;
use std::{net::IpAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub host: IpAddr,
    pub port: u16,
    pub token: String,
}

#[derive(Clone)]
struct AppState {
    auth: Auth,
    server: Arc<Server>,
}

pub async fn serve(workspace: Workspace, options: ServerOptions) -> Result<()> {
    let app = app(workspace, options.token);
    let listener = TcpListener::bind((options.host, options.port))
        .await
        .with_context(|| {
            format!(
                "failed to bind MCP HTTP server to {}:{}",
                options.host, options.port
            )
        })?;
    eprintln!(
        "neopad-mcp listening on http://{}/mcp",
        listener.local_addr()?
    );
    axum::serve(listener, app)
        .await
        .context("MCP HTTP server failed")
}

fn app(workspace: Workspace, token: String) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp))
        .with_state(AppState {
            auth: Auth::new(token),
            server: Arc::new(Server::new(workspace)),
        })
}

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    if let Err(error) = state.auth.validate(&headers) {
        return error.into_response();
    }

    let response = state.server.handle_request(request);
    if response.is_null() {
        StatusCode::ACCEPTED.into_response()
    } else {
        Json(response).into_response()
    }
}

#[allow(dead_code)]
fn plain(status: StatusCode, message: &'static str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .expect("response")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{header, Method, Request, StatusCode},
    };
    use neopad_core::init_workspace;
    use serde_json::json;
    use tower::ServiceExt;

    fn test_app() -> (tempfile::TempDir, Router) {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        (temp, app(workspace, "secret".to_owned()))
    }

    #[tokio::test]
    async fn initialize_succeeds_with_valid_token() {
        let (_temp, app) = test_app();
        let response = app
            .oneshot(mcp_request(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize"
            })))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let value: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(value["result"]["serverInfo"]["name"], "neopad-mcp");
    }

    #[tokio::test]
    async fn request_without_token_is_rejected() {
        let (_temp, app) = test_app();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/mcp")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_is_rejected() {
        let (_temp, app) = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mcp")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, "Bearer wrong")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn suspicious_origin_is_rejected() {
        let (_temp, app) = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mcp")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, "Bearer secret")
                    .header(header::ORIGIN, "https://example.com")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn tools_list_exposes_write_tools() {
        let (_temp, app) = test_app();
        let response = app
            .oneshot(mcp_request(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            })))
            .await
            .expect("response");
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let value: Value = serde_json::from_slice(&body).expect("json");
        let tools = value["result"]["tools"].as_array().expect("tools");

        assert!(tools.iter().any(|tool| tool["name"] == "append_to_inbox"));
        assert!(tools.iter().any(|tool| tool["name"] == "trash_page"));
        assert!(!tools.iter().any(|tool| tool["name"] == "delete_page"));
    }

    fn mcp_request(value: Value) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/mcp")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, "Bearer secret")
            .body(Body::from(value.to_string()))
            .expect("request")
    }
}

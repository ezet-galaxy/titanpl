use std::{collections::HashMap, fs, sync::Arc, path::PathBuf};

use anyhow::Result;
use axum::{
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Json},
    routing::any,
    Router,
};
use boa_engine::{Context, Source};
use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;

// ----------------------
// Route structures
// ----------------------
#[derive(Debug, Deserialize)]
struct RouteVal {
    r#type: String,
    value: Value,
}

#[derive(Clone)]
struct AppState {
    routes: Arc<HashMap<String, RouteVal>>,
    project_root: PathBuf,     // FIXED: replaces server_dir
}

// ----------------------
// Root / Dynamic handler
// ----------------------
async fn root_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}
async fn dynamic_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}

// ----------------------
// Main dynamic handler
// ----------------------
async fn dynamic_handler_inner(
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    let method = req.method().as_str().to_uppercase();
    let path = req.uri().path();
    let key = format!("{}:{}", method, path);

    let body_bytes = match to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read body").into_response(),
    };
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    if let Some(route) = state.routes.get(&key) {
        match route.r#type.as_str() {

            // --------------------------
            // ACTION ROUTE
            // --------------------------
            "action" => {
                let action_name = route.value.as_str().unwrap_or("").trim();
                if action_name.is_empty() {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid action name",
                    ).into_response();
                }

                // correct action path (bundle path)
                let action_path = state
                    .project_root
                    .join("server")
                    .join("actions")
                    .join(format!("{}.jsbundle", action_name));

                if !action_path.exists() {
                    return (
                        StatusCode::NOT_FOUND,
                        format!("Action bundle not found: {:?}", action_path),
                    )
                    .into_response();
                }

                // read JS bundle
                let js_code = match fs::read_to_string(action_path) {
                    Ok(v) => v,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed reading action bundle: {}", e),
                        ).into_response()
                    }
                };

                // inject request
                let injected = format!(
                    "const __titan_req = {};\n{};\n{}(__titan_req);",
                    body_str,
                    js_code,
                    action_name
                );                

                // exec in Boa
                let mut ctx = Context::default();
                let result = match ctx.eval(Source::from_bytes(&injected)) {
                    Ok(v) => v,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("JS execution error: {}", e.to_string()),
                        )
                            .into_response();
                    }
                };

                // convert JsValue -> JSON (Boa returns Option<Value>)
                let result_json: Value = match result.to_json(&mut ctx) {
                    Ok(Some(v)) => v,
                    Ok(None) => serde_json::json!({ "error": "JS returned undefined" }),
                    Err(e) => json_error(e.to_string()),
                };

                

                return Json(result_json).into_response();
            }

            // --------------------------
            // STATIC JSON
            // --------------------------
            "json" => return Json(route.value.clone()).into_response(),

            // --------------------------
            // TEXT
            // --------------------------
            _ => {
                if let Some(s) = route.value.as_str() {
                    return s.to_string().into_response();
                }
                return route.value.to_string().into_response();
            }
        }
    }

    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

fn json_error(msg: String) -> Value {
    serde_json::json!({ "error": msg })
}

// ----------------------
// MAIN
// ----------------------
#[tokio::main]
async fn main() -> Result<()> {
    let raw = fs::read_to_string("./routes.json").unwrap_or_else(|_| "{}".to_string());
    let json: Value = serde_json::from_str(&raw).unwrap_or_default();

    let port = json["__config"]["port"].as_u64().unwrap_or(3000);

    let routes_json = json["routes"].clone();
    let map: HashMap<String, RouteVal> =
        serde_json::from_value(routes_json).unwrap_or_default();

        let project_root = std::env::current_dir()?
        .parent()
        .unwrap()
        .to_path_buf();

    let state = AppState {
        routes: Arc::new(map),
        project_root,
    };

    // router
    let app = Router::new()
        .route("/", any(root_route))
        .fallback(any(dynamic_route))
        .with_state(state);

    // run
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    //
    // TITAN BANNER
    //
    println!(
        "\n\x1b[38;5;208m\
████████╗██╗████████╗ █████╗ ███╗   ██╗\n\
╚══██╔══╝██║╚══██╔══╝██╔══██╗████╗  ██║\n\
   ██║   ██║   ██║   ███████║██╔██╗ ██║\n\
   ██║   ██║   ██║   ██╔══██║██║╚██╗██║\n\
   ██║   ██║   ██║   ██║  ██║██║ ╚████║\n\
   ╚═╝   ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝\x1b[0m\n"
    );

    println!(
        "\x1b[38;5;39mTitan server running at:\x1b[0m \x1b[97mhttp://localhost:{}\x1b[0m\n",
        port
    );
    axum::serve(listener, app).await?;
    Ok(())
}

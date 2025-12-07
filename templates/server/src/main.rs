// src/main.rs
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

use boa_engine::{
    Context, JsValue, Source,
    native_function::NativeFunction,
    object::ObjectInitializer,
    property::Attribute,
};
use boa_engine::js_string;

use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::task;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Route configuration entry parsed from routes.json
#[derive(Debug, Deserialize)]
struct RouteVal {
    r#type: String,
    value: Value,
}

#[derive(Clone)]
struct AppState {
    routes: Arc<HashMap<String, RouteVal>>,
    project_root: PathBuf,
}

/// Inject a synchronous `t.fetch(url, opts?)` into the Boa context.
/// This `t.fetch` runs the blocking HTTP call inside `tokio::task::block_in_place`
/// so it is safe to call while inside an async Tokio context.
fn inject_t_fetch(ctx: &mut Context) {
    // Create native Rust function (Boa v0.20)
    let t_fetch_native = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        // Extract arguments (safely convert JS strings to owned Rust Strings)
        let url = args
            .get(0)
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_default();

        // opts may be undefined. Convert to serde_json::Value for thread-safety.
        let opts_js = args.get(1).cloned().unwrap_or(JsValue::undefined());
        let opts_json: Value = match opts_js.to_json(ctx) {
            Ok(v) => v,
            Err(_) => Value::Object(serde_json::Map::new()),
        };

        // Extract method, body, headers from opts_json (owned data, Send)
        let method = opts_json
            .get("method")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "GET".to_string());

        let body_opt = match opts_json.get("body") {
            Some(Value::String(s)) => Some(s.clone()),
            Some(other) => Some(other.to_string()),
            None => None,
        };

        // Build header map from opts_json["headers"] if present
        let mut header_pairs: Vec<(String, String)> = Vec::new();
        if let Some(Value::Object(map)) = opts_json.get("headers") {
            for (k, v) in map.iter() {
                let v_str = match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                header_pairs.push((k.clone(), v_str));
            }
        }

        // Perform blocking HTTP request inside block_in_place so we don't drop a blocking runtime
        let out_json = task::block_in_place(move || {
            // Create blocking client
            let client = Client::new();

            // Build request
            let method_parsed = method.parse().unwrap_or(reqwest::Method::GET);
            let mut req = client.request(method_parsed, &url);

            // Attach headers
            if !header_pairs.is_empty() {
                let mut headers = HeaderMap::new();
                for (k, v) in header_pairs.into_iter() {
                    if let (Ok(name), Ok(val)) =
                        (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(&v))
                    {
                        headers.insert(name, val);
                    }
                }
                req = req.headers(headers);
            }

            if let Some(body) = body_opt {
                req = req.body(body);
            }

            // Send request
            match req.send() {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    // Try to read text, fallback to empty string on error
                    let text = resp.text().unwrap_or_default();
                    serde_json::json!({
                        "ok": true,
                        "status": status,
                        "body": text
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "ok": false,
                        "error": e.to_string()
                    })
                }
            }
        });

        // Convert serde_json::Value -> JsValue for return to JS
        Ok(JsValue::from_json(&out_json, ctx).unwrap_or(JsValue::undefined()))
    });

    // Convert the native function into a JS function object (requires Realm in Boa 0.20)
    let realm = ctx.realm();
    let t_fetch_js_fn = t_fetch_native.to_js_function(realm);

    // Build `t` object with `.fetch` property
    let t_obj = ObjectInitializer::new(ctx)
        .property(js_string!("fetch"), t_fetch_js_fn, Attribute::all())
        .build();

    // Attach to globalThis.t
    ctx.global_object()
        .set(js_string!("t"), JsValue::from(t_obj), false, ctx)
        .unwrap();
}

// Axum handlers --------------------------------------------------------------

async fn root_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}
async fn dynamic_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}

/// Main handler that evaluates JS actions from bundles using Boa
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
            "action" => {
                let action_name = route.value.as_str().unwrap_or("").trim();
                if action_name.is_empty() {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid action name",
                    )
                        .into_response();
                }

                let action_path = state.project_root
                    .join("actions")
                    .join(format!("{}.jsbundle", action_name));



                if !action_path.exists() {
                    return (
                        StatusCode::NOT_FOUND,
                        format!("Action bundle not found: {:?}", action_path),
                    )
                        .into_response();
                }

                let js_code = match fs::read_to_string(action_path) {
                    Ok(v) => v,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed reading action bundle: {}", e),
                        )
                            .into_response();
                    }
                };

                // Build env
                let mut env_map = serde_json::Map::new();
                for (k, v) in std::env::vars() {
                    env_map.insert(k, Value::String(v));
                }
                let env_json = Value::Object(env_map);

                // Injected JS: set process.env, provide request payload, then eval bundle and call action
                let injected = format!(
                    r#"
                    globalThis.process = {{ env: {} }};
                    const __titan_req = {};
                    {};
                    {}(__titan_req);
                    "#,
                    env_json.to_string(),
                    body_str,
                    js_code,
                    action_name
                );

                // Create Boa context, inject t.fetch, evaluate
                let mut ctx = Context::default();
                inject_t_fetch(&mut ctx);

                let result = match ctx.eval(Source::from_bytes(&injected)) {
                    Ok(v) => v,
                    Err(e) => return Json(json_error(e.to_string())).into_response(),
                };

                // to_json returns Result<Value, JsError> in Boa 0.20
                let result_json: Value = match result.to_json(&mut ctx) {
                    Ok(v) => v,
                    Err(e) => json_error(e.to_string()),
                };

                return Json(result_json).into_response();
            }

            "json" => return Json(route.value.clone()).into_response(),
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

// Entrypoint -----------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

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

    let app = Router::new()
        .route("/", any(root_route))
        .fallback(any(dynamic_route))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

//
    // TITAN BANNER
    //
    println!("\n\x1b[38;5;208m████████╗██╗████████╗ █████╗ ███╗   ██╗");
    println!("╚══██╔══╝██║╚══██╔══╝██╔══██╗████╗  ██║");
    println!("   ██║   ██║   ██║   ███████║██╔██╗ ██║");
    println!("   ██║   ██║   ██║   ██╔══██║██║╚██╗██║");
    println!("   ██║   ██║   ██║   ██║  ██║██║ ╚████║");
    println!("   ╚═╝   ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝\x1b[0m\n");

    println!("\x1b[38;5;39mTitan server running at:\x1b[0m http://localhost:{}", port);
    
    axum::serve(listener, app).await?;
    Ok(())
}

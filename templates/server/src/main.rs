use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use anyhow::Result;
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Json},
    routing::any,
    Router,
};

use boa_engine::{Context, Source};
use serde_json::Value;
use tokio::net::TcpListener;
use std::time::Instant;

mod utils;
mod errors;
mod extensions;
mod action_management;

use utils::{blue, white, yellow, green, gray, red};
use errors::format_js_error;
use extensions::inject_t_runtime;
use action_management::{
    resolve_actions_dir, find_actions_dir, match_dynamic_route, 
    DynamicRoute, RouteVal
};

#[derive(Clone)]
struct AppState {
    routes: Arc<HashMap<String, RouteVal>>,
    dynamic_routes: Arc<Vec<DynamicRoute>>,
    project_root: PathBuf,
}

// Root/dynamic handlers -----------------------------------------------------

async fn root_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}

async fn dynamic_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}

/// Main handler: looks up routes.json and executes action bundles using Boa.
async fn dynamic_handler_inner(
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {

    // ---------------------------
    // BASIC REQUEST INFO
    // ---------------------------
    let method = req.method().as_str().to_uppercase();
    let path = req.uri().path().to_string();
    let key = format!("{}:{}", method, path);

    // ---------------------------
    // TIMER + LOG META
    // ---------------------------
    let start = Instant::now();
    let mut route_label = String::from("not_found");
    let mut route_kind = "none"; // exact | dynamic | reply

    // ---------------------------
    // QUERY PARSING
    // ---------------------------
    let query: HashMap<String, String> = req
        .uri()
        .query()
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut it = pair.splitn(2, '=');
                    Some((
                        it.next()?.to_string(),
                        it.next().unwrap_or("").to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    // ---------------------------
    // HEADERS & BODY
    // ---------------------------
    let (parts, body) = req.into_parts();
    
    let headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect::<HashMap<String, String>>();

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "Failed to read request body",
            )
                .into_response()
        }
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let body_json: Value = if body_str.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&body_str).unwrap_or(Value::String(body_str))
    };

    // ---------------------------
    // ROUTE RESOLUTION
    // ---------------------------
    let mut params: HashMap<String, String> = HashMap::new();
    let mut action_name: Option<String> = None;

    // Exact route
    if let Some(route) = state.routes.get(&key) {
        route_kind = "exact";

        if route.r#type == "action" {
            let name = route.value.as_str().unwrap_or("unknown").to_string();
            route_label = name.clone();
            action_name = Some(name);
        } else if route.r#type == "json" {
            let elapsed = start.elapsed();
            println!(
                "{} {} {} {}",
                blue("[Titan]"),
                white(&format!("{} {}", method, path)),
                white("→ json"),
                gray(&format!("in {:.2?}", elapsed))
            );
            return Json(route.value.clone()).into_response();
        } else if let Some(s) = route.value.as_str() {
            let elapsed = start.elapsed();
            println!(
                "{} {} {} {}",
                blue("[Titan]"),
                white(&format!("{} {}", method, path)),
                white("→ reply"),
                gray(&format!("in {:.2?}", elapsed))
            );
            return s.to_string().into_response();
        }
    }

    // Dynamic route
    if action_name.is_none() {
        if let Some((action, p)) =
            match_dynamic_route(&method, &path, state.dynamic_routes.as_slice())
        {
            route_kind = "dynamic";
            route_label = action.clone();
            action_name = Some(action);
            params = p;
        }
    }

    let action_name = match action_name {
        Some(a) => a,
        None => {
            let elapsed = start.elapsed();
            println!(
                "{} {} {} {}",
                blue("[Titan]"),
                white(&format!("{} {}", method, path)),
                white("→ 404"),
                gray(&format!("in {:.2?}", elapsed))
            );
            return (StatusCode::NOT_FOUND, "Not Found").into_response();
        }
    };

    // ---------------------------
    // LOAD ACTION
    // ---------------------------
    let resolved = resolve_actions_dir();
    let actions_dir = resolved
        .exists()
        .then(|| resolved)
        .or_else(|| find_actions_dir(&state.project_root))
        .unwrap();

    let action_path = actions_dir.join(format!("{}.jsbundle", action_name));
    let js_code = match fs::read_to_string(&action_path) {
        Ok(c) => c,
        Err(_) => {
             // Handle missing bundle gracefully
             return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Action bundle not found",
                    "action": action_name
                })),
            ).into_response()
        }
    };

    // ---------------------------
    // ENV
    // ---------------------------
    let env_json = std::env::vars()
        .map(|(k, v)| (k, Value::String(v)))
        .collect::<serde_json::Map<_, _>>();



    // ---------------------------
    // JS EXECUTION
    // ---------------------------
    let injected = format!(
        r#"
        globalThis.process = {{ env: {} }};
        const __titan_req = {{
            body: {},
            method: "{}",
            path: "{}",
            headers: {},
            params: {},
            query: {}
        }};
        {};
        globalThis["{}"](__titan_req);
        "#,
        Value::Object(env_json).to_string(),
        body_json.to_string(),
        method,
        path,
        serde_json::to_string(&headers).unwrap(),
        serde_json::to_string(&params).unwrap(),
        serde_json::to_string(&query).unwrap(),
        js_code,
        action_name
    );

    let mut ctx = Context::default();
    inject_t_runtime(&mut ctx, &action_name, &state.project_root);
    let result = match ctx.eval(Source::from_bytes(&injected)) {
        Ok(v) => v,
        Err(err) => {
            let elapsed = start.elapsed();
    
            let details = format_js_error(err, &route_label);
    
            println!(
                "{} {} {} {}",
                blue("[Titan]"),
                red(&format!("{} {}", method, path)),
                red("→ error"),
                gray(&format!("in {:.2?}", elapsed))
            );
    
            println!("{}", red(&details));
    
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Action execution failed",
                    "action": route_label,
                    "details": details
                })),
            )
                .into_response();
        }
    };
    
    let result_json: Value = if result.is_undefined() {
        Value::Null
    } else {
        match result.to_json(&mut ctx) {
            Ok(v) => v,
            Err(err) => {
                let elapsed = start.elapsed();
                println!(
                    "{} {} {} {}",
                    blue("[Titan]"),
                    red(&format!("{} {}", method, path)),
                    red("→ serialization error"),
                    gray(&format!("in {:.2?}", elapsed))
                );
    
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to serialize action result",
                        "details": err.to_string()
                    })),
                )
                    .into_response();
            }
        }
    };
    
    
    // ---------------------------
    // FINAL LOG
    // ---------------------------
    let elapsed = start.elapsed();
    match route_kind {
        "dynamic" => println!(
            "{} {} {} {} {} {}",
            blue("[Titan]"),
            green(&format!("{} {}", method, path)),
            white("→"),
            green(&route_label),
            white("(dynamic)"),
            gray(&format!("in {:.2?}", elapsed))
        ),
        "exact" => println!(
            "{} {} {} {} {}",
            blue("[Titan]"),
            white(&format!("{} {}", method, path)),
            white("→"),
            yellow(&route_label),
            gray(&format!("in {:.2?}", elapsed))
        ),
        _ => {}
    }

    Json(result_json).into_response()
}


// Entrypoint ---------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Load routes.json (expected at runtime root)
    let raw = fs::read_to_string("./routes.json").unwrap_or_else(|_| "{}".to_string());
    let json: Value = serde_json::from_str(&raw).unwrap_or_default();

    let port = json["__config"]["port"].as_u64().unwrap_or(3000);
    let routes_json = json["routes"].clone();
    let map: HashMap<String, RouteVal> =
    serde_json::from_value(routes_json).unwrap_or_default();

    let dynamic_routes: Vec<DynamicRoute> =
    serde_json::from_value(json["__dynamic_routes"].clone())
        .unwrap_or_default();

    // Project root — heuristics: try current_dir()
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let state = AppState {
        routes: Arc::new(map),
        dynamic_routes: Arc::new(dynamic_routes),
        project_root,
    };
    

    let app = Router::new()
        .route("/", any(root_route))
        .fallback(any(dynamic_route))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    // Banner (yellow-orange) and server info
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

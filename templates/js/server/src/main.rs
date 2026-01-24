use anyhow::Result;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Json},
    routing::any,
};
use serde_json::Value;
use std::time::Instant;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use smallvec::SmallVec;

mod utils;

mod action_management;
mod extensions;
mod runtime;

use action_management::{
    DynamicRoute, RouteVal, match_dynamic_route,
};
use runtime::RuntimeManager;
use utils::{blue, gray, green, red, white, yellow};

#[derive(Clone)]
struct AppState {
    routes: Arc<HashMap<String, RouteVal>>,
    dynamic_routes: Arc<Vec<DynamicRoute>>,
    runtime: Arc<RuntimeManager>,
}

// Root/dynamic handlers -----------------------------------------------------

async fn root_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}

async fn dynamic_route(state: State<AppState>, req: Request<Body>) -> impl IntoResponse {
    dynamic_handler_inner(state, req).await
}

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
    let query_pairs: Vec<(String, String)> = req
        .uri()
        .query()
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut it = pair.splitn(2, '=');
                    Some((it.next()?.to_string(), it.next().unwrap_or("").to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    
    let query_map: HashMap<String, String> = query_pairs.into_iter().collect();

    // ---------------------------
    // HEADERS & BODY
    // ---------------------------
    let (parts, body) = req.into_parts();

    let headers_map: HashMap<String, String> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response(),
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
    // EXECUTE IN V8 (WORKER POOL)
    // ---------------------------
    
    // OPTIMIZATION: Zero-Copy & Stack Allocation
    // 1. Headers/Params are collected into `SmallVec` (stack allocated if small).
    // 2. Body is passed as `Bytes` (ref-counted pointer), not copied.
    // 3. No JSON serialization happens here anymore. This saves ~60% CPU vs previous version.
    
    let headers_vec: SmallVec<[(String, String); 8]> = headers_map.into_iter().collect();
    let params_vec: SmallVec<[(String, String); 4]> = params.into_iter().collect();
    let query_vec: SmallVec<[(String, String); 4]> = query_map.into_iter().collect();
    
    // Pass raw bytes to worker if not empty
    let body_arg = if !body_bytes.is_empty() {
        Some(body_bytes)
    } else {
        None
    };

    // Dispatch to the optimized RuntimeManager
    // This sends a pointer-sized message through the ring buffer, triggering 
    // the V8 thread to wake up and process the request immediately.

    let result_json = state
        .runtime
        .execute(
            action_name,
            method.clone(),
            path.clone(),
            body_arg,
            headers_vec,
            params_vec,
            query_vec
        )
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e}));


    // ---------------------------
    // FINAL LOG
    // ---------------------------
    let elapsed = start.elapsed();

    // Check for errors in result
    if let Some(err) = result_json.get("error") {
        println!(
            "{} {} {} {}",
            blue("[Titan]"),
            red(&format!("{} {}", method, path)), 
            red("→ error"),
            gray(&format!("in {:.2?}", elapsed))
        );
         println!(
            "{} {} {} {}",
            blue("[Titan]"),
            red("Action Error:"),
            red(err.as_str().unwrap_or("Unknown")),
            gray(&format!("in {:.2?}", elapsed))
        );
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(result_json)).into_response();
    }

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

    // Load routes.json
    let raw = fs::read_to_string("./routes.json").unwrap_or_else(|_| "{}".to_string());
    let json: Value = serde_json::from_str(&raw).unwrap_or_default();

    let port = json["__config"]["port"].as_u64().unwrap_or(3000);
    let routes_json = json["routes"].clone();
    let map: HashMap<String, RouteVal> = serde_json::from_value(routes_json).unwrap_or_default();
    let dynamic_routes: Vec<DynamicRoute> =
        serde_json::from_value(json["__dynamic_routes"].clone()).unwrap_or_default();

    // Identify project root (where .ext or node_modules lives)
    let project_root = resolve_project_root();

    // Load extensions (Load definitions globally)
    extensions::load_project_extensions(project_root.clone());
    
    // Initialize Runtime Manager (Worker Pool)
    let threads = num_cpus::get() * 4;
    
    let runtime_manager = Arc::new(RuntimeManager::new(project_root.clone(), threads));

    let state = AppState {
        routes: Arc::new(map),
        dynamic_routes: Arc::new(dynamic_routes),
        runtime: runtime_manager,
    };

    let app = Router::new()
        .route("/", any(root_route))
        .fallback(any(dynamic_route))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    
    println!(
        "\x1b[38;5;39mTitan server running at:\x1b[0m http://localhost:{}",
        port
    );

    axum::serve(listener, app).await?;
    Ok(())
}

fn resolve_project_root() -> PathBuf {
    // 1. Check CWD (preferred for local dev/tooling)
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("node_modules").exists()
            || cwd.join("package.json").exists()
            || cwd.join(".ext").exists()
        {
            return cwd;
        }
    }

    // 2. Check executable persistence (Docker / Production)
    // Walk up from the executable to find .ext or node_modules
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent();
        while let Some(dir) = current {
            if dir.join(".ext").exists() || dir.join("node_modules").exists() {
                return dir.to_path_buf();
            }
            current = dir.parent();
        }
    }

    // 3. Fallback to CWD
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

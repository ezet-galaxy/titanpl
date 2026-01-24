#![allow(unused)]
pub mod builtin;
pub mod external;

use v8;
use std::sync::Once;
use std::path::PathBuf;
use std::sync::{Mutex, Arc, OnceLock};
use std::collections::HashMap;
use std::fs;
use dashmap::DashMap;
use tokio::sync::broadcast;
use crate::action_management::scan_actions;
use bytes::Bytes;
use crossbeam::channel::Sender;
use serde_json::Value;
use crate::utils::{blue, red, gray, green};

// ----------------------------------------------------------------------------
// GLOBALS
// ----------------------------------------------------------------------------

pub static SHARE_CONTEXT: OnceLock<ShareContextStore> = OnceLock::new();

pub struct ShareContextStore {
    pub kv: DashMap<String, serde_json::Value>,
    pub broadcast_tx: broadcast::Sender<(String, serde_json::Value)>,
}

impl ShareContextStore {
    pub fn get() -> &'static Self {
        SHARE_CONTEXT.get_or_init(|| {
            let (tx, _) = broadcast::channel(1000);
            Self {
                kv: DashMap::new(),
                broadcast_tx: tx,
            }
        })
    }
}

// Re-exports for easier access
pub use external::load_project_extensions;

// ----------------------------------------------------------------------------
// TITAN RUNTIME
// ----------------------------------------------------------------------------

pub struct TitanRuntime {
    pub isolate: v8::OwnedIsolate,
    pub context: v8::Global<v8::Context>,
    pub actions: HashMap<String, v8::Global<v8::Function>>,
    pub worker_tx: crossbeam::channel::Sender<crate::runtime::WorkerCommand>,
}

unsafe impl Send for TitanRuntime {}
unsafe impl Sync for TitanRuntime {}

static V8_INIT: Once = Once::new();

pub fn init_v8() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

pub fn init_runtime_worker(root: PathBuf, worker_tx: crossbeam::channel::Sender<crate::runtime::WorkerCommand>) -> TitanRuntime {
    init_v8();
    
    // Memory optimization strategy (v8 0.106.0 limitations):
    // - V8 snapshots reduce memory footprint by sharing compiled code
    // - Each isolate still has its own heap, but the snapshot reduces base overhead
    // - For explicit heap limits, use V8 flags: --max-old-space-size=128
    
    let params = v8::CreateParams::default();
    let mut isolate = v8::Isolate::new(params);
    
    let (global_context, actions_map) = {
        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(handle_scope, v8::ContextOptions::default());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let global = context.global(scope);
        
        // Inject Titan Runtime APIs
        inject_extensions(scope, global);

        // Root Metadata (Dynamic per app instance)
        let root_str = v8::String::new(scope, root.to_str().unwrap_or(".")).unwrap();
        let root_key = v8_str(scope, "__titan_root");
        global.set(scope, root_key.into(), root_str.into());

        // Load Actions (Cold start optimization target)
        let mut map = HashMap::new();
        let action_files = scan_actions(&root);
        for (name, path) in action_files {
             if let Ok(code) = fs::read_to_string(&path) {
                 let wrapped_source = format!("(function() {{ {} }})(); globalThis[\"{}\"];", code, name);
                 let source_str = v8_str(scope, &wrapped_source);
                 let try_catch = &mut v8::TryCatch::new(scope);
                 if let Some(script) = v8::Script::compile(try_catch, source_str, None) {
                     if let Some(val) = script.run(try_catch) {
                         if val.is_function() {
                             let func = v8::Local::<v8::Function>::try_from(val).unwrap();
                             map.insert(name.clone(), v8::Global::new(try_catch, func));
                         }
                     }
                 }
             }
        }
        (v8::Global::new(scope, context), map)
    };

    TitanRuntime {
        isolate,
        context: global_context,
        actions: actions_map,
        worker_tx,
    }
}

pub fn inject_extensions(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) {
    // Ensuring globalThis
    let gt_key = v8_str(scope, "globalThis");
    global.set(scope, gt_key.into(), global.into());

    let t_obj = v8::Object::new(scope);
    let t_key = v8_str(scope, "t");
    global.create_data_property(scope, t_key.into(), t_obj.into()).unwrap();

    // Call individual injectors
    builtin::inject_builtin_extensions(scope, global, t_obj);
    external::inject_external_extensions(scope, global, t_obj);
    
    // Inject t.db (Stub)
    let db_obj = v8::Object::new(scope);
    let db_key = v8_str(scope, "db");
    t_obj.set(scope, db_key.into(), db_obj.into());

    global.set(scope, t_key.into(), t_obj.into());
}

// ----------------------------------------------------------------------------
// EXECUTION HELPERS
// ----------------------------------------------------------------------------

pub fn execute_action_optimized(
    runtime: &mut TitanRuntime,
    action_name: &str, 
    req_body: Option<bytes::Bytes>, 
    req_method: &str, 
    req_path: &str, 
    headers: &[(String, String)], 
    params: &[(String, String)], 
    query: &[(String, String)]
) -> serde_json::Value {
    let TitanRuntime { isolate, context: global_context, actions: actions_map, .. } = runtime;
    let handle_scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Local::new(handle_scope, &*global_context);
    let scope = &mut v8::ContextScope::new(handle_scope, context);
    
    let req_obj = v8::Object::new(scope);
    
    let m_key = v8_str(scope, "method");
    let m_val = v8_str(scope, req_method);
    req_obj.set(scope, m_key.into(), m_val.into());
    
    let p_key = v8_str(scope, "path");
    let p_val = v8_str(scope, req_path);
    req_obj.set(scope, p_key.into(), p_val.into());

    let body_val: v8::Local<v8::Value> = if let Some(bytes) = req_body {
        let vec = bytes.to_vec();
        let store = v8::ArrayBuffer::new_backing_store_from_boxed_slice(vec.into_boxed_slice());
        let ab = v8::ArrayBuffer::with_backing_store(scope, &store.make_shared());
        ab.into()
    } else { v8::null(scope).into() };
    let rb_key = v8_str(scope, "rawBody");
    req_obj.set(scope, rb_key.into(), body_val);

    let h_obj = v8::Object::new(scope);
    for (k, v) in headers { 
        let k_v8 = v8_str(scope, k);
        let v_v8 = v8_str(scope, v);
        h_obj.set(scope, k_v8.into(), v_v8.into()); 
    }
    let h_key = v8_str(scope, "headers");
    req_obj.set(scope, h_key.into(), h_obj.into());

    let p_obj = v8::Object::new(scope);
    for (k, v) in params { 
        let k_v8 = v8_str(scope, k);
        let v_v8 = v8_str(scope, v);
        p_obj.set(scope, k_v8.into(), v_v8.into()); 
    }
    let params_key = v8_str(scope, "params");
    req_obj.set(scope, params_key.into(), p_obj.into());

    let q_obj = v8::Object::new(scope);
    for (k, v) in query { 
        let k_v8 = v8_str(scope, k);
        let v_v8 = v8_str(scope, v);
        q_obj.set(scope, k_v8.into(), v_v8.into()); 
    }
    let q_key = v8_str(scope, "query");
    req_obj.set(scope, q_key.into(), q_obj.into());

    let global = context.global(scope);
    let req_tr_key = v8_str(scope, "__titan_req");
    global.set(scope, req_tr_key.into(), req_obj.into());

    if let Some(action_global) = actions_map.get(action_name) {
        let action_fn = v8::Local::new(scope, action_global);
        let tr_act_key = v8_str(scope, "__titan_action");
        let tr_act_val = v8_str(scope, action_name);
        global.set(scope, tr_act_key.into(), tr_act_val.into());
        let try_catch = &mut v8::TryCatch::new(scope);
        if let Some(result) = action_fn.call(try_catch, global.into(), &[req_obj.into()]) {
             if let Some(json) = v8::json::stringify(try_catch, result) {
                 return serde_json::from_str(&json.to_rust_string_lossy(try_catch)).unwrap_or(serde_json::Value::Null);
             }
        }
        let msg = try_catch.message().map(|m| m.get(try_catch).to_rust_string_lossy(try_catch)).unwrap_or("Unknown error".to_string());
        return serde_json::json!({"error": msg});
    }
    serde_json::json!({"error": format!("Action '{}' not found", action_name)})
}

pub fn v8_str<'s>(scope: &mut v8::HandleScope<'s>, s: &str) -> v8::Local<'s, v8::String> {
    v8::String::new(scope, s).unwrap()
}

pub fn v8_to_string(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> String {
    value.to_string(scope).unwrap().to_rust_string_lossy(scope)
}

pub fn throw(scope: &mut v8::HandleScope, msg: &str) {
    let message = v8_str(scope, msg);
    let exception = v8::Exception::error(scope, message);
    scope.throw_exception(exception);
}

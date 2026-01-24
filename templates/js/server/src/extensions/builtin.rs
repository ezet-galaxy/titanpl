use v8;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::utils::{blue, gray, parse_expires_in};
use super::{TitanRuntime, v8_str, v8_to_string, throw, ShareContextStore};

const TITAN_CORE_JS: &str = include_str!("titan_core.js");

pub fn inject_builtin_extensions(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>, t_obj: v8::Local<v8::Object>) {
    // 1. Native API Bindings
    
    // defineAction (Native side)
    let def_fn = v8::Function::new(scope, native_define_action).unwrap();
    let def_key = v8_str(scope, "defineAction");
    global.set(scope, def_key.into(), def_fn.into());

    
    // t.read
    let read_fn = v8::Function::new(scope, native_read).unwrap();
    let read_key = v8_str(scope, "read");
    t_obj.set(scope, read_key.into(), read_fn.into());

    // t.decodeUtf8
    let dec_fn = v8::Function::new(scope, native_decode_utf8).unwrap();
    let dec_key = v8_str(scope, "decodeUtf8");
    t_obj.set(scope, dec_key.into(), dec_fn.into());

    // t.log
    let log_fn = v8::Function::new(scope, native_log).unwrap();
    let log_key = v8_str(scope, "log");
    t_obj.set(scope, log_key.into(), log_fn.into());
    
    // t.fetch
    let fetch_fn = v8::Function::new(scope, native_fetch).unwrap();
    let fetch_key = v8_str(scope, "fetch");
    t_obj.set(scope, fetch_key.into(), fetch_fn.into());

    // auth, jwt, password ... (keep native)
    setup_native_utils(scope, t_obj);

    // 2. JS Side Injection (Embedded)
    let source = v8_str(scope, TITAN_CORE_JS);
    if let Some(script) = v8::Script::compile(scope, source, None) {
        script.run(scope);
    }
}

fn setup_native_utils(scope: &mut v8::HandleScope, t_obj: v8::Local<v8::Object>) {
    // t.jwt
    let jwt_obj = v8::Object::new(scope);
    let sign_fn = v8::Function::new(scope, native_jwt_sign).unwrap();
    let verify_fn = v8::Function::new(scope, native_jwt_verify).unwrap();
    
    let sign_key = v8_str(scope, "sign");
    jwt_obj.set(scope, sign_key.into(), sign_fn.into());
    let verify_key = v8_str(scope, "verify");
    jwt_obj.set(scope, verify_key.into(), verify_fn.into());
    
    let jwt_key = v8_str(scope, "jwt");
    t_obj.set(scope, jwt_key.into(), jwt_obj.into());

    // t.password
    let pw_obj = v8::Object::new(scope);
    let hash_fn = v8::Function::new(scope, native_password_hash).unwrap();
    let pw_verify_fn = v8::Function::new(scope, native_password_verify).unwrap();
    
    let hash_key = v8_str(scope, "hash");
    pw_obj.set(scope, hash_key.into(), hash_fn.into());
    let pw_v_key = v8_str(scope, "verify");
    pw_obj.set(scope, pw_v_key.into(), pw_verify_fn.into());
    
    let pw_key = v8_str(scope, "password");
    t_obj.set(scope, pw_key.into(), pw_obj.into());

    // t.shareContext (Native primitives)
    let sc_obj = v8::Object::new(scope);
    let n_get = v8::Function::new(scope, share_context_get).unwrap();
    let n_set = v8::Function::new(scope, share_context_set).unwrap();
    let n_del = v8::Function::new(scope, share_context_delete).unwrap();
    let n_keys = v8::Function::new(scope, share_context_keys).unwrap();
    let n_pub = v8::Function::new(scope, share_context_broadcast).unwrap();

    let get_key = v8_str(scope, "get");
    sc_obj.set(scope, get_key.into(), n_get.into());
    let set_key = v8_str(scope, "set");
    sc_obj.set(scope, set_key.into(), n_set.into());
    let del_key = v8_str(scope, "delete");
    sc_obj.set(scope, del_key.into(), n_del.into());
    let keys_key = v8_str(scope, "keys");
    sc_obj.set(scope, keys_key.into(), n_keys.into());
    let pub_key = v8_str(scope, "broadcast");
    sc_obj.set(scope, pub_key.into(), n_pub.into());
    
    let sc_key = v8_str(scope, "shareContext");
    let sc_val = sc_obj.into();
    t_obj.set(scope, sc_key.into(), sc_val);
}

fn native_read(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let path_val = args.get(0);
    if !path_val.is_string() {
        throw(scope, "t.read(path): path is required");
        return;
    }
    let path_str = v8_to_string(scope, path_val);

    if std::path::Path::new(&path_str).is_absolute() {
        throw(scope, "t.read expects a relative path like 'db/file.sql'");
        return;
    }

    let context = scope.get_current_context();
    let global = context.global(scope);
    let root_key = v8_str(scope, "__titan_root");
    let root_val = global.get(scope, root_key.into()).unwrap();
    
    let root_str = if root_val.is_string() {
        v8_to_string(scope, root_val)
    } else {
        throw(scope, "Internal Error: __titan_root not set");
        return;
    };

    let root_path = PathBuf::from(root_str);
    let root_path = root_path.canonicalize().unwrap_or(root_path);
    let joined = root_path.join(&path_str);

    let target = match joined.canonicalize() {
        Ok(t) => t,
        Err(_) => {
            throw(scope, &format!("t.read: file not found: {}", path_str));
            return;
        }
    };

    if !target.starts_with(&root_path) {
        throw(scope, "t.read: path escapes allowed root");
        return;
    }

    match std::fs::read_to_string(&target) {
        Ok(content) => {
            retval.set(v8_str(scope, &content).into());
        },
        Err(e) => {
            throw(scope, &format!("t.read failed: {}", e));
        }
    }
}

fn native_decode_utf8(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let val = args.get(0);
    if let Ok(u8arr) = v8::Local::<v8::Uint8Array>::try_from(val) {
        let buf = u8arr.buffer(scope).unwrap();
        let store = v8::ArrayBuffer::get_backing_store(&buf);
        let offset = usize::from(u8arr.byte_offset());
        let length = usize::from(u8arr.byte_length());
        let slice = &store[offset..offset+length];
        
        let bytes: Vec<u8> = slice.iter().map(|b| b.get()).collect();
        let s = String::from_utf8_lossy(&bytes);
        retval.set(v8_str(scope, &s).into());
    } else if let Ok(ab) = v8::Local::<v8::ArrayBuffer>::try_from(val) {
        let store = v8::ArrayBuffer::get_backing_store(&ab);
        let bytes: Vec<u8> = store.iter().map(|b| b.get()).collect();
        let s = String::from_utf8_lossy(&bytes);
        retval.set(v8_str(scope, &s).into());
    } else {
        retval.set(v8::null(scope).into());
    }
}

fn share_context_get(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let key = v8_to_string(scope, args.get(0));
    let store = ShareContextStore::get();
    if let Some(val) = store.kv.get(&key) {
        let json_str = val.to_string();
        let v8_str = v8::String::new(scope, &json_str).unwrap();
        if let Some(v8_val) = v8::json::parse(scope, v8_str) {
            retval.set(v8_val);
            return;
        }
    }
    retval.set(v8::null(scope).into());
}

fn share_context_set(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue) {
    let key = v8_to_string(scope, args.get(0));
    let val_v8 = args.get(1);
    
    if let Some(json_v8) = v8::json::stringify(scope, val_v8) {
        let json_str = json_v8.to_rust_string_lossy(scope);
        if let Ok(val) = serde_json::from_str(&json_str) {
            ShareContextStore::get().kv.insert(key, val);
        }
    }
}

fn share_context_delete(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue) {
    let key = v8_to_string(scope, args.get(0));
    ShareContextStore::get().kv.remove(&key);
}

fn share_context_keys(scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let store = ShareContextStore::get();
    let keys: Vec<v8::Local<v8::Value>> = store.kv.iter().map(|kv| v8_str(scope, kv.key()).into()).collect();
    let arr = v8::Array::new_with_elements(scope, &keys);
    retval.set(arr.into());
}

fn share_context_broadcast(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue) {
    let event = v8_to_string(scope, args.get(0));
    let payload_v8 = args.get(1);
    
    if let Some(json_v8) = v8::json::stringify(scope, payload_v8) {
        let json_str = json_v8.to_rust_string_lossy(scope);
        if let Ok(payload) = serde_json::from_str(&json_str) {
            let _ = ShareContextStore::get().broadcast_tx.send((event, payload));
        }
    }
}



fn native_log(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _retval: v8::ReturnValue) {
    let context = scope.get_current_context();
    let global = context.global(scope);
    let action_key = v8_str(scope, "__titan_action");
    let action_name = if let Some(action_val) = global.get(scope, action_key.into()) {
        if action_val.is_string() {
            v8_to_string(scope, action_val)
        } else {
            "init".to_string()
        }
    } else {
        "init".to_string()
    };

    let mut parts = Vec::new();
    for i in 0..args.length() {
        let val = args.get(i);
        let mut appended = false;
        
        if val.is_object() && !val.is_function() {
             if let Some(json) = v8::json::stringify(scope, val) {
                 parts.push(json.to_rust_string_lossy(scope));
                 appended = true;
             }
        }
        
        if !appended {
            parts.push(v8_to_string(scope, val));
        }
    }
    
    let titan_str = blue("[Titan]");
    let log_msg = gray(&format!("\x1b[90mlog({})\x1b[0m\x1b[97m: {}\x1b[0m", action_name, parts.join(" ")));
    println!(
        "{} {}",
        titan_str,
        log_msg
    );
}

fn native_fetch(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let url = v8_to_string(scope, args.get(0));
    let mut method = "GET".to_string();
    let mut body_str = None;
    let mut headers_vec = Vec::new();

    let opts_val = args.get(1);
    if opts_val.is_object() {
        let opts_obj = opts_val.to_object(scope).unwrap();
        
        let m_key = v8_str(scope, "method");
        if let Some(m_val) = opts_obj.get(scope, m_key.into()) {
            if m_val.is_string() {
                method = v8_to_string(scope, m_val);
            }
        }
        
        let b_key = v8_str(scope, "body");
        if let Some(b_val) = opts_obj.get(scope, b_key.into()) {
            if b_val.is_string() {
                body_str = Some(v8_to_string(scope, b_val));
            } else if b_val.is_object() {
                 let json_obj = v8::json::stringify(scope, b_val).unwrap();
                 body_str = Some(json_obj.to_rust_string_lossy(scope));
            }
        }
        
        let h_key = v8_str(scope, "headers");
        if let Some(h_val) = opts_obj.get(scope, h_key.into()) {
            if h_val.is_object() {
                let h_obj = h_val.to_object(scope).unwrap();
                if let Some(keys) = h_obj.get_own_property_names(scope, Default::default()) {
                    for i in 0..keys.length() {
                        let key = keys.get_index(scope, i).unwrap();
                        let val = h_obj.get(scope, key).unwrap();
                        headers_vec.push((v8_to_string(scope, key), v8_to_string(scope, val)));
                    }
                }
            }
        }
    }

    let client = Client::builder().use_rustls_tls().tcp_nodelay(true).build().unwrap_or(Client::new());
    let mut req = client.request(method.parse().unwrap_or(reqwest::Method::GET), &url);
    
    for (k, v) in headers_vec {
        if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(&v)) {
            let mut map = HeaderMap::new();
            map.insert(name, val);
            req = req.headers(map);
        }
    }
    
    if let Some(b) = body_str {
        req = req.body(b);
    }
    
    let res = req.send();
    let obj = v8::Object::new(scope);
    match res {
        Ok(r) => {
            let status = r.status().as_u16();
            let text = r.text().unwrap_or_default();
            
            let status_key = v8_str(scope, "status");
            let status_val = v8::Number::new(scope, status as f64);
            obj.set(scope, status_key.into(), status_val.into());
            
            let body_key = v8_str(scope, "body");
            let body_val = v8_str(scope, &text);
            obj.set(scope, body_key.into(), body_val.into());
            
            let ok_key = v8_str(scope, "ok");
            let ok_val = v8::Boolean::new(scope, true);
            obj.set(scope, ok_key.into(), ok_val.into());
        }, 
        Err(e) => {
            let ok_key = v8_str(scope, "ok");
            let ok_val = v8::Boolean::new(scope, false);
            obj.set(scope, ok_key.into(), ok_val.into());
            
            let err_key = v8_str(scope, "error");
            let err_val = v8_str(scope, &e.to_string());
            obj.set(scope, err_key.into(), err_val.into());
        }
    }
    retval.set(obj.into());
}

fn native_jwt_sign(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let payload_val = args.get(0);
    let json_str = v8::json::stringify(scope, payload_val).unwrap().to_rust_string_lossy(scope);
    let mut payload: serde_json::Map<String, Value> = serde_json::from_str(&json_str).unwrap_or_default();
    let secret = v8_to_string(scope, args.get(1));
    
    let opts_val = args.get(2);
    if opts_val.is_object() {
        let opts_obj = opts_val.to_object(scope).unwrap();
        let exp_key = v8_str(scope, "expiresIn");
        if let Some(val) = opts_obj.get(scope, exp_key.into()) {
             let seconds = if val.is_number() {
                 Some(val.to_number(scope).unwrap().value() as u64)
             } else if val.is_string() {
                 parse_expires_in(&v8_to_string(scope, val))
             } else { None };
             if let Some(sec) = seconds {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                payload.insert("exp".to_string(), Value::Number(serde_json::Number::from(now + sec)));
             }
        }
    }

    let token = encode(&Header::default(), &Value::Object(payload), &EncodingKey::from_secret(secret.as_bytes()));
    match token {
        Ok(t) => {
            let res = v8_str(scope, &t);
            retval.set(res.into());
        },
        Err(e) => throw(scope, &e.to_string()),
    }
}

fn native_jwt_verify(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let token = v8_to_string(scope, args.get(0));
    let secret = v8_to_string(scope, args.get(1));
    let mut validation = Validation::default();
    validation.validate_exp = true;
    let data = decode::<Value>(&token, &DecodingKey::from_secret(secret.as_bytes()), &validation);
    match data {
        Ok(d) => {
             let json_str = serde_json::to_string(&d.claims).unwrap();
             let v8_json_str = v8_str(scope, &json_str);
             if let Some(val) = v8::json::parse(scope, v8_json_str) {
                 retval.set(val);
             }
        },
        Err(e) => throw(scope, &format!("Invalid or expired JWT: {}", e)),
    }
}

fn native_password_hash(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let pw = v8_to_string(scope, args.get(0));
    match hash(pw, DEFAULT_COST) {
        Ok(h) => {
            let res = v8_str(scope, &h);
            retval.set(res.into());
        },
        Err(e) => throw(scope, &e.to_string()),
    }
}

fn native_password_verify(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    let pw = v8_to_string(scope, args.get(0));
    let hash_str = v8_to_string(scope, args.get(1));
    let ok = verify(pw, &hash_str).unwrap_or(false);
    retval.set(v8::Boolean::new(scope, ok).into());
}

fn native_define_action(_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut retval: v8::ReturnValue) {
    retval.set(args.get(0));
}

use boa_engine::{
    js_string, native_function::NativeFunction, object::ObjectInitializer, property::Attribute,
    Context, JsError, JsValue,
};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use bcrypt::{hash, verify, DEFAULT_COST};
use postgres::{Client as PgClient, NoTls};
use postgres::types::{ToSql, Type};
use std::path::PathBuf;

use crate::utils::{blue, gray, parse_expires_in};

/// Here add all the runtime t base things
/// Injects a synchronous `t.fetch(url, opts?)` function into the Boa `Context`.
pub fn inject_t_runtime(ctx: &mut Context, action_name: &str, project_root: &PathBuf) {

    // =========================================================
    // t.read(path)
    // =========================================================
    let root = project_root.clone();
    let t_read_native = unsafe { NativeFunction::from_closure(move |_this, args, ctx| {
        let path_str = args.get(0)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .ok_or_else(|| {
                JsError::from_native(
                    boa_engine::JsNativeError::typ().with_message("t.read(path): path is required")
                )
            })?;
            
        let target_path = root.join(path_str);
        
        // Security check? For now assuming trusted code.
        // We should arguably check if it's inside project_root, but dev tool.
        
        let content = std::fs::read_to_string(target_path)
            .map_err(|e| JsError::from_native(
                boa_engine::JsNativeError::error().with_message(format!("Failed to read file: {}", e))
            ))?;
            
        Ok(JsValue::from(js_string!(content)))
    })};

    // =========================================================
    // t.log(...)  — unsafe by design (Boa requirement)
    // =========================================================
    let action = action_name.to_string();

    let t_log_native = unsafe {
        NativeFunction::from_closure(move |_this, args, _ctx| {
            let mut parts = Vec::new();

            for arg in args {
                parts.push(arg.display().to_string());
            }

            println!(
                "{} {}",
                blue("[Titan]"),
                gray(&format!("\x1b[90mlog({})\x1b[0m\x1b[97m: {}\x1b[0m", action, parts.join(" ")))
            );

            Ok(JsValue::undefined())
        })
    };

    // =========================================================
    // t.fetch(...) — no capture, safe fn pointer
    // =========================================================
    let t_fetch_native = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        // -----------------------------
        // 1. URL (required)
        // -----------------------------
        let url = match args.get(0) {
            Some(v) => v.to_string(ctx)?.to_std_string_escaped(),
            None => {
                return Err(JsError::from_native(
                    boa_engine::JsNativeError::typ()
                        .with_message("t.fetch(url[, options]): url is required"),
                ));
            }
        };

        // -----------------------------
        // 2. Options
        // -----------------------------
        let opts_js = args.get(1).cloned().unwrap_or(JsValue::Null);

        let opts_json = match opts_js.to_json(ctx) {
            Ok(v) => v,
            Err(_) => Value::Object(serde_json::Map::new()),
        };

        let method = opts_json
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("GET")
            .to_string();

            let body_opt = opts_json.get("body").map(|v| {
                if v.is_string() {
                    v.as_str().unwrap().to_string()
                } else {
                    serde_json::to_string(v).unwrap_or_default()
                }
            });
            

        let mut header_pairs = Vec::new();
        if let Some(Value::Object(map)) = opts_json.get("headers") {
            for (k, v) in map {
                if let Some(val) = v.as_str() {
                    header_pairs.push((k.clone(), val.to_string()));
                }
            }
        }

        // -----------------------------
        // 3. Blocking HTTP (safe fallback)
        // -----------------------------
        let out_json = task::block_in_place(move || {
            let client = Client::builder()
           .use_rustls_tls()
            .tcp_nodelay(true)
            .build()
            .unwrap();


            let mut req = client.request(method.parse().unwrap_or(reqwest::Method::GET), &url);

            if !header_pairs.is_empty() {
                let mut headers = HeaderMap::new();
                for (k, v) in header_pairs {
                    if let (Ok(name), Ok(val)) = (
                        HeaderName::from_bytes(k.as_bytes()),
                        HeaderValue::from_str(&v),
                    ) {
                        headers.insert(name, val);
                    }
                }
                req = req.headers(headers);
            }

            if let Some(body) = body_opt {
                req = req.body(body);
            }

            match req.send() {
                Ok(resp) => serde_json::json!({
                    "ok": true,
                    "status": resp.status().as_u16(),
                    "body": resp.text().unwrap_or_default()
                }),
                Err(e) => serde_json::json!({
                    "ok": false,
                    "error": e.to_string()
                }),
            }
        });

        // -----------------------------
        // 4. JSON → JsValue (NO undefined fallback)
        // -----------------------------
        match JsValue::from_json(&out_json, ctx) {
            Ok(v) => Ok(v),
            Err(e) => Err(boa_engine::JsNativeError::error()
                .with_message(format!("t.fetch: JSON conversion failed: {}", e))
                .into()),
        }
    });

    // =========================================================
    // t.jwt
    // =========================================================
    let t_jwt_sign = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        // payload (must be object)
        let mut payload = args.get(0)
            .and_then(|v| v.to_json(ctx).ok())
            .and_then(|v| v.as_object().cloned())
            .ok_or_else(|| {
                JsError::from_native(
                    boa_engine::JsNativeError::typ()
                        .with_message("t.jwt.sign(payload, secret[, options])"),
                )
            })?;
    
        // secret
        let secret = args.get(1)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
    

        if let Some(opts) = args.get(2) {
            if let Ok(Value::Object(opts)) = opts.to_json(ctx) {
                if let Some(exp) = opts.get("expiresIn") {
                    let seconds = match exp {
                        Value::Number(n) => n.as_u64(),
                        Value::String(s) => parse_expires_in(s),
                     _ => None,
                    };

            if let Some(sec) = seconds {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                payload.insert(
                    "exp".to_string(),
                    Value::Number(serde_json::Number::from(now + sec)),
                );
            }
        }
    }
}

        let token = encode(
            &Header::default(),
            &Value::Object(payload),
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| {
            JsError::from_native(
                boa_engine::JsNativeError::error().with_message(e.to_string()),
            )
        })?;
    
        Ok(JsValue::from(js_string!(token)))
    });
    
    let t_jwt_verify = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let token = args.get(0)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
    
        let secret = args.get(1)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
    
        let mut validation = Validation::default();
        validation.validate_exp = true;
    
        let data = decode::<Value>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map_err(|_| {
            JsError::from_native(
                boa_engine::JsNativeError::error()
                    .with_message("Invalid or expired JWT"),
            )
        })?;
    
        JsValue::from_json(&data.claims, ctx).map_err(|e| e.into())
    });
    

    
    // =========================================================
    // t.password
    // =========================================================
    let t_password_hash = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let password = args.get(0)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
    
        let hashed = hash(password, DEFAULT_COST)
            .map_err(|e| JsError::from_native(
                boa_engine::JsNativeError::error().with_message(e.to_string())
            ))?;
    
            Ok(JsValue::from(js_string!(hashed)))
    });

    let t_password_verify = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let password = args.get(0)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
    
        let hash_str = args.get(1)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default();
    
        let ok = verify(password, &hash_str).unwrap_or(false);
    
        Ok(JsValue::from(ok))
    });

    // =========================================================
    // t.db (Synchronous Postgres)
    // =========================================================
    let t_db_connect = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let url = args.get(0)
            .and_then(|v| v.to_string(ctx).ok())
            .map(|s| s.to_std_string_escaped())
            .ok_or_else(|| {
                JsError::from_native(
                    boa_engine::JsNativeError::typ()
                        .with_message("t.db.connect(url): url string is required"),
                )
            })?;

       
        
        let url_clone = url.clone();
        
        let query_fn = unsafe {
            NativeFunction::from_closure(move |_this, args, ctx| {
                let sql = args.get(0)
                    .and_then(|v| v.to_string(ctx).ok())
                    .map(|s| s.to_std_string_escaped())
                    .ok_or_else(|| {
                        JsError::from_native(
                            boa_engine::JsNativeError::typ().with_message("db.query(sql, params): sql is required")
                        )
                    })?;

                
                let params_val = args.get(1).cloned().unwrap_or(JsValue::Null);
                
               
                let json_params: Vec<Value> = if let Ok(val) = params_val.to_json(ctx) {
                   if let Value::Array(arr) = val { arr } else { vec![] }
                } else {
                   vec![]
                };
                
                let url_for_query = url_clone.clone();

                let result_json = task::block_in_place(move || {
                     let mut client = match PgClient::connect(&url_for_query, NoTls) {
                        Ok(c) => c,
                        Err(e) => return Err(format!("Connection failed: {}", e)),
                     };
                     
                     // We need to map `Vec<Value>` to `&[&dyn ToSql]`.
                     
                     let mut typed_params: Vec<Box<dyn ToSql + Sync>> = Vec::new();
                     
                     for p in json_params {
                         match p {
                             Value::String(s) => typed_params.push(Box::new(s)),
                             Value::Number(n) => {
                                 if let Some(i) = n.as_i64() {
                                     typed_params.push(Box::new(i));
                                 } else if let Some(f) = n.as_f64() {
                                     typed_params.push(Box::new(f));
                                 }
                             },
                             Value::Bool(b) => typed_params.push(Box::new(b)),
                             Value::Null => typed_params.push(Box::new(Option::<String>::None)), // Typed null?
                             // Fallback others to JSON
                             obj => typed_params.push(Box::new(obj)),
                         }
                     }
                     
                     let param_refs: Vec<&(dyn ToSql + Sync)> = typed_params
                        .iter()
                        .map(|x| x.as_ref())
                        .collect();
                        
                     let rows = client.query(&sql, &param_refs).map_err(|e| e.to_string())?;
                     
                     // Convert rows to JSON
                     let mut out_rows = Vec::new();
                     for row in rows {
                         let mut map = serde_json::Map::new();
                         // We need column names.
                         for (i, col) in row.columns().iter().enumerate() {
                             let name = col.name().to_string();
                             
                             let val: Value = match *col.type_() {
                                 Type::BOOL => Value::Bool(row.get(i)),
                                 Type::INT2 | Type::INT4 | Type::INT8 => {
                                      let v: Option<i64> = row.get::<_, Option<i64>>(i);
                                      v.map(|n| Value::Number(n.into())).unwrap_or(Value::Null)
                                 },
                                 Type::FLOAT4 | Type::FLOAT8 => {
                                      let v: Option<f64> = row.get::<_, Option<f64>>(i);
                                      v.map(|n| serde_json::Number::from_f64(n).map(Value::Number).unwrap_or(Value::Null)).unwrap_or(Value::Null)
                                 },
                                 Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
                                     let v: Option<String> = row.get(i);
                                     v.map(Value::String).unwrap_or(Value::Null)
                                 },
                                 Type::JSON | Type::JSONB => {
                                     let v: Option<Value> = row.get(i);
                                     v.unwrap_or(Value::Null)
                                 },
                                 _ => Value::Null 
                             };
                             map.insert(name, val);
                         }
                         out_rows.push(Value::Object(map));
                     }
                     
                     Ok(out_rows)
                });
                
                match result_json {
                    Ok(rows) => JsValue::from_json(&Value::Array(rows), ctx),
                    Err(e) => Err(JsError::from_native(boa_engine::JsNativeError::error().with_message(e)))
                }
            })
        };
        
        // Build object
        
        let realm = ctx.realm().clone(); // Fix context borrow
        let obj = ObjectInitializer::new(ctx)
            .property(
                js_string!("query"),
                query_fn.to_js_function(&realm),
                Attribute::all(),
            )
            .build();
            
        Ok(JsValue::from(obj))
    });

   
    // =========================================================
    // Build global `t`
    // =========================================================
    let realm = ctx.realm().clone();

    let jwt_obj = ObjectInitializer::new(ctx)
    .property(js_string!("sign"), t_jwt_sign.to_js_function(&realm), Attribute::all())
    .property(js_string!("verify"), t_jwt_verify.to_js_function(&realm), Attribute::all())
    .build();

    let password_obj = ObjectInitializer::new(ctx)
    .property(js_string!("hash"), t_password_hash.to_js_function(&realm), Attribute::all())
    .property(js_string!("verify"), t_password_verify.to_js_function(&realm), Attribute::all())
    .build();
    
    let db_obj = ObjectInitializer::new(ctx)
    .property(js_string!("connect"), t_db_connect.to_js_function(&realm), Attribute::all())
    .build();

    let t_obj = ObjectInitializer::new(ctx)
        .property(
            js_string!("log"),
            t_log_native.to_js_function(&realm),
            Attribute::all(),
        )
        .property(
            js_string!("fetch"),
            t_fetch_native.to_js_function(&realm),
            Attribute::all(),
        )    
        .property(
            js_string!("read"),
            t_read_native.to_js_function(&realm),
            Attribute::all()
        )
        .property(js_string!("jwt"), jwt_obj, Attribute::all())
        .property(js_string!("password"), password_obj, Attribute::all())
        .property(js_string!("db"), db_obj, Attribute::all())
        .build();

    ctx.global_object()
        .set(js_string!("t"), JsValue::from(t_obj), false, ctx)
        .expect("set global t");
}

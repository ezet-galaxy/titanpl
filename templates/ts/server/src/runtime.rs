use std::thread;
use crossbeam::channel::{bounded, Sender};
use tokio::sync::oneshot;
use bytes::Bytes;
use smallvec::SmallVec;
use crate::extensions;

// ----------------------------------------------------------------------------
// TITANVM: HIGH-PERFORMANCE WORKER POOL
// ----------------------------------------------------------------------------

/// The command sent from the Async Axum thread to the Sync V8 Worker thread.
/// 
/// IMPLEMENTATION NOTE: Zero-Copy Design
/// Instead of passing `String` or `Vec<u8>` which incur heap allocations for every request,
/// we use:
/// 1. `Bytes`: An Arc-counted slice of the original TCP buffer. Cloning this is O(1).
/// 2. `SmallVec`: Stack-allocated vectors for headers/params. 99% of requests fit in standard limits
///    (8 headers, 4 params), avoiding malloc/free overhead entirely.
pub enum WorkerCommand {
    Request(RequestTask),
}

pub struct RequestTask {
    pub action_name: String,
    
    // Zero-copy body (Arc-based byte slice)
    // This slice points directly into the Hyper/Tokio TCP buffer.
    // It is passed to V8 as an ArrayBuffer BackingStore without copying.
    pub body: Option<Bytes>, 
    
    // Efficient Metadata (No JSON)
    pub method: String,
    pub path: String,
    
    // SmallVec<[T; N]> stores N items inline on the struct (stack memory).
    // Only unnecessary heap allocation occurs if headers > 8.
    pub headers: SmallVec<[(String, String); 8]>,
    pub params: SmallVec<[(String, String); 4]>,
    pub query: SmallVec<[(String, String); 4]>,

    // Response channel
    // Used to signal the Async Runtime when the Sync V8 work is done.
    pub response_tx: oneshot::Sender<WorkerResult>,
}


pub struct WorkerResult {
    pub json: serde_json::Value,
}

pub struct RuntimeManager {
    sender: Sender<WorkerCommand>,
    _workers: Vec<thread::JoinHandle<()>>,
}

impl RuntimeManager {
    pub fn new(project_root: std::path::PathBuf, num_threads: usize) -> Self {
        let (tx, rx) = bounded::<WorkerCommand>(num_threads * 2000); 
        
        let mut workers = Vec::new();
        
        for i in 0..num_threads {
            let rx_clone = rx.clone();
            let tx_clone = tx.clone();
            let root_clone = project_root.clone();
            
            let handle = thread::Builder::new()
                .name(format!("titan-worker-{}", i))
                .spawn(move || {
                    // 1. Thread-Local Event Loop Init
                    // Initialize independent V8 Isolate for this thread
                    let mut runtime = extensions::init_runtime_worker(root_clone, tx_clone);
                    
                    // 2. Event Loop
                    let runtime_ptr = &mut runtime as *mut extensions::TitanRuntime as *mut std::ffi::c_void;
                    runtime.isolate.set_data(0, runtime_ptr);

                    loop {
                        // Strictly synchronous blocking receive
                        // This makes the worker a dedicated processor for requests from the queue.
                        // No background event loop is running.
                        if let Ok(cmd) = rx_clone.recv() {
                            handle_cmd(cmd, &mut runtime);
                        } else {
                            break;
                        }
                    }
                })
                .expect("Failed to spawn worker thread");
            
            workers.push(handle);
        }

        Self {
            sender: tx,
            _workers: workers,
        }
    }

    // Optimized Execute method (Takes maps/vecs instead of JSON strings)
    pub async fn execute(
        &self, 
        action: String, 
        method: String, 
        path: String, 
        body: Option<Bytes>,
        headers: SmallVec<[(String, String); 8]>,
        params: SmallVec<[(String, String); 4]>,
        query: SmallVec<[(String, String); 4]>,
    ) -> Result<serde_json::Value, String> {
        let (tx, rx) = oneshot::channel();
        
        let task = RequestTask {
            action_name: action,
            body,
            method,
            path,
            headers,
            params,
            query,
            response_tx: tx,
        };
        
        // Dispatch to RingBuffer/Channel
        self.sender.send(WorkerCommand::Request(task)).map_err(|e| e.to_string())?;
        
        // Await Result (Async-Sync Bridge)
        match rx.await {
            Ok(res) => Ok(res.json),
            Err(_) => Err("Worker channel closed".to_string()),
        }
    }

}

fn handle_cmd(cmd: WorkerCommand, runtime: &mut extensions::TitanRuntime) {

     match cmd {
         WorkerCommand::Request(task) => {
             // 3. Execution (Zero-Copy)
             let result = extensions::execute_action_optimized(
                runtime,
                &task.action_name,
                task.body,
                &task.method,
                &task.path,
                &task.headers,
                &task.params,
                &task.query
            );
            
            let _ = task.response_tx.send(WorkerResult {
                json: result,
            });
         }
     }
}

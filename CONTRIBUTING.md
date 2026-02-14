# Contributing to Titan Planet 🪐

We love contributions! Titan is a unique hybrid framework that combines **Rust's performance** with **JavaScript/TypeScript's productivity**. There are opportunities for contributors from both ecosystems.

---

## 🛠 Prerequisites

Before you start, ensure you have:
- **Node.js** (v18+)
- **Rust** (Latest stable from [rustup](https://rustup.rs/))
- **Titan CLI** installed globally: `npm install -g @ezetgalaxy/titan`
- **Git** for version control
- **A code editor** (VS Code recommended with Rust Analyzer extension)

---

## 🟡 For JavaScript/TypeScript Developers

You can contribute to the **CLI**, **Templates**, **Extensions**, and **Documentation**.

### 1. The CLI (`index.js`)

The Titan CLI orchestrates project scaffolding, dev server management, and build coordination.

**Location**: `index.js`, `scripts/`

**Areas to contribute:**
- **Project Scaffolding** (`titan init`):
  - Improve template selection UX
  - Add new project templates or starter kits
  - Enhance interactive prompts
  
- **Dev Server** (`titan dev`):
  - Improve file watching and hot-reload logic (chokidar)
  - Enhance log formatting and error reporting
  - Add performance monitoring for build times
  
- **Bundling Pipeline**:
  - Optimize esbuild configuration
  - Add support for new bundling strategies
  - Improve TypeScript type checking integration
  
- **New Commands**:
  - `titan dockerfile` - Generate optimized Dockerfiles
  - `titan benchmark` - Built-in performance testing
  - `titan migrate` - Migration tools for breaking changes

**Example PR ideas:**
```javascript
// Add better error recovery in the dev server
if (buildError) {
  console.error(formatError(buildError));
  // Don't kill the server, wait for fix
  return;
}
```

### 2. Templates (`templates/`)

Titan ships with four isolated project templates: `js`, `ts`, `rust-js`, `rust-ts`.

**Location**: `templates/`

**Areas to contribute:**
- **Example Actions**: Add real-world examples (auth, payments, file uploads)
- **Type Definitions**: Improve `titan.d.ts` for better IntelliSense
- **Default Routes**: Create sensible `routes.json` defaults
- **Documentation**: Add inline code comments explaining patterns

**Template structure:**
```
templates/
├── common/           # Shared across all templates
├── js/              # Pure JavaScript template
├── ts/              # Pure TypeScript template
├── rust-js/         # Hybrid Rust + JS template
└── rust-ts/         # Hybrid Rust + TS template
```

### 3. Extensions System

Build powerful extensions without touching the core!

**Create a new extension:**
```bash
titan create ext my-feature
cd my-feature
npm install
titan run ext
```

**Extension capabilities:**
- Native Rust bindings
- Custom Titan APIs (e.g., `t.myFeature.doSomething()`)
- Type-safe TypeScript definitions
- WebAssembly-like ABI for high-performance operations

**Example extensions to build:**
- `titanpl-redis` - Redis client
- `titanpl-postgres` - PostgreSQL driver
- `titanpl-s3` - AWS S3 integration
- `titanpl-websocket` - WebSocket support

### 4. The Incubator (`incubator/`)

The `incubator` folder is our **testing ground** for new features, architectural experiments, and patterns before they become official templates.

**Location**: `incubator/`

**Workflow**:
1.  **Experiment**: Build prototypes or test breaking changes in `incubator/js` or `incubator/rust`.
2.  **Validate**: Verify stability and performance in isolation.
3.  **Promote**: Once polished, migrate the changes to the official `templates/` directory.

Use this space for rapid prototyping without affecting stable releases.

### 5. Documentation

Help make Titan more accessible!

**Areas to contribute:**
- **Tutorials**: Step-by-step guides for common tasks
- **API Reference**: Complete coverage of `t` namespace
- **Migration Guides**: From Node.js, Deno, Bun
- **Performance Guides**: Optimization best practices
- **Video Content**: Screen recordings, tutorials

---

## 🔴 For Rust Developers

You can contribute to the **Core Server**, **V8 Runtime**, **Performance**, and **Native Extensions**.

### 1. Core Server (`templates/*/server/`)

The heart of Titan: **Axum + V8 + Multi-threaded workers**.

**Location**: `templates/*/server/src/`

**Key files and areas:**

#### `main.rs` - HTTP Server & Request Routing
- **Optimize Axum routing**: Add middleware support, improve route matching
- **Request parsing**: Enhance JSON parsing, add support for multipart/form-data
- **Error handling**: Improve error responses and logging
- **Graceful shutdown**: Better signal handling

#### `runtime.rs` - Worker Pool Management
- **Worker lifecycle**: Improve initialization and shutdown
- **Load balancing**: Implement smarter request distribution
- **Health checks**: Add worker health monitoring
- **Dynamic scaling**: Auto-scale worker pool based on load

#### `extensions/mod.rs` - V8 Orchestration
- **Isolate management**: Optimize V8 isolate creation and reuse
- **Context initialization**: Reduce cold start time
- **Memory management**: Implement better heap limits
- **Snapshot support**: Add V8 snapshot creation for faster startup

**Current architecture:**
```rust
// Worker pool pattern
let (tx, rx) = crossbeam_channel::unbounded();
for _ in 0..num_workers {
    let rx_clone = rx.clone();
    thread::spawn(move || {
        let mut runtime = TitanRuntime::new();
        loop {
            let task = rx_clone.recv().unwrap();
            let response = runtime.execute_action(&task);
            task.respond(response);
        }
    });
}
```

#### `extensions/builtin.rs` - First-Party APIs
- **Add new APIs**: Database, caching, sessions, cookies
- **Performance**: Optimize existing APIs (`t.fetch`, `t.jwt`)
- **Error handling**: Better V8 exception mapping

**Example: Adding a new API**
```rust
fn native_database_query(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // 1. Extract arguments from V8
    let query = v8_to_string(scope, args.get(0));
    
    // 2. Execute native Rust operation
    let result = execute_query(&query);
    
    // 3. Convert back to V8 value
    let v8_result = v8::String::new(scope, &result).unwrap();
    rv.set(v8_result.into());
}
```

#### `extensions/external.rs` - Extension Loader
- **Dynamic loading**: Improve extension discovery from `node_modules`
- **ABI engine**: Enhance type marshaling between JS and Rust
- **Security**: Add sandboxing for untrusted extensions

### 2. Performance Optimizations

**Areas to contribute:**

#### Cold Start Optimization
- **V8 Snapshots**: Implement proper `SnapshotCreator` usage
- **Lazy loading**: Defer non-critical initialization
- **Binary size**: Reduce compiled binary size

Current target: **~3-5ms** → **<2ms**

#### Memory Efficiency
- **Heap limits**: Expose `ResourceConstraints` API
- **Shared code**: Reduce per-worker memory overhead
- **Garbage collection**: Tune V8 GC parameters

Current: **~40-80MB/worker** → Target: **<30MB/worker**

#### Request Throughput
- **Zero-copy parsing**: Avoid JSON serialization where possible
- **Object reuse**: Reduce V8 object allocation
- **JIT optimization**: Ensure hot paths are JIT-friendly

Current: **~10k req/sec** → Target: **>15k req/sec**

### 3. Platform Support

**Help make Titan truly cross-platform:**

- **Windows**: Fix file locking issues, improve process management
- **Linux**: Optimize for musl/glibc, improve systemd integration
- **macOS**: Resolve Apple Silicon-specific issues
- **Docker**: Optimize Alpine/Debian images for smaller size
- **Cross-compilation**: Improve build scripts for different targets

### 4. Testing & Reliability

**Improve test coverage:**

- **Unit tests**: Core runtime functions, API handlers
- **Integration tests**: End-to-end action execution
- **Performance tests**: Benchmark critical paths
- **Fuzz testing**: Find edge cases in V8 bindings

**Example test structure:**
```rust
#[test]
fn test_action_execution() {
    let runtime = TitanRuntime::new();
    let request = json!({ "path": "/test", "method": "GET" });
    let response = runtime.execute_action("test", &request);
    assert_eq!(response.status, 200);
}
```

---

## 🚀 Development Workflow

### Initial Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/ezet-galaxy/titanpl.git
   cd titanpl
   npm install
   ```

2. **Link CLI locally**
   ```bash
   npm link
   ```
   Now `titan` commands use your local development version.

3. **Create a test project**
   ```bash
   mkdir test-app
   cd test-app
   titan init
   # Select your template
   titan dev
   ```

### Making Changes

**For JavaScript/CLI changes:**
```bash
# Edit index.js or templates/
# Test with a local project
cd test-app
titan dev # Uses your local CLI
```

**For Rust server changes:**
```bash
# Navigate to a template's server
cd templates/js/server

# Make changes to src/

# Build and test
cargo build --release
cargo test

# Or use titan dev to test in context
cd ../../..
cd test-app
titan dev # Rebuilds the Rust server
```

### Testing Your Changes

**JavaScript Testing:**
```bash
npm test              # Run all tests
npm run test:watch   # Watch mode
npm run test:e2e     # End-to-end tests
```

**Rust Testing:**
```bash
cd templates/*/server
cargo test            # Run unit tests
cargo test -- --nocapture  # Show output
cargo bench          # Run benchmarks
```

**Manual Testing:**
```bash
# Create test projects for each template
titan init test-js --template js
titan init test-ts --template ts
titan init test-rust-js --template rust-js
titan init test-rust-ts --template rust-ts

# Test each one
cd test-js && titan dev
```

---

## 🎨 Code Style Guidelines

### JavaScript/TypeScript
- **ES6+ syntax**: Use modern JavaScript features
- **No semicolons**: Follow standard JS style (except where required)
- **Descriptive names**: `handleActionRequest` not `doStuff`
- **Comments**: Explain _why_, not _what_
- **Type safety**: Use JSDoc or TypeScript where applicable

**Example:**
```javascript
/**
 * Executes a JavaScript action within the V8 isolate.
 * @param {string} actionName - The name of the action to execute
 * @param {object} request - The incoming HTTP request data
 * @returns {Promise<object>} The action response
 */
async function executeAction(actionName, request) {
    // Implementation
}
```

### Rust
- **Run `cargo fmt`** before committing
- **Run `cargo clippy`** to catch common issues
- **Follow Rust naming conventions**: `snake_case` for functions, `PascalCase` for types
- **Avoid `unwrap()`**: Use proper error handling (`?`, `Result<T, E>`)
- **Document public APIs**: Use `///` doc comments

**Example:**
```rust
/// Executes a JavaScript action within the V8 isolate.
///
/// # Arguments
/// * `action_name` - The name of the action to execute
/// * `request` - The incoming HTTP request data
///
/// # Returns
/// The JSON response from the action
pub fn execute_action(action_name: &str, request: &Value) -> Result<Value, Error> {
    // Implementation
}
```

---

## 📋 Pull Request Guidelines

### Before Submitting

1. **Create an issue first** for major features/changes
2. **Follow the code style** guidelines above
3. **Write tests** for new functionality
4. **Update documentation** if you change behavior
5. **Test all four templates** (js, ts, rust-js, rust-ts)

### PR Checklist

- [ ] Code follows style guidelines (JS: standard, Rust: `cargo fmt`)
- [ ] All tests pass (`npm test` and `cargo test`)
- [ ] New features have tests
- [ ] Documentation is updated (README, CHANGELOG, inline comments)
- [ ] Tested on all relevant templates
- [ ] No breaking changes (or clearly documented)
- [ ] Commit messages are clear and descriptive

### Commit Message Format

```
[area] Brief description

Longer explanation if needed.

Fixes #issue-number
```

**Examples:**
```
[cli] Add dockerfile generation command

Implements `titan dockerfile` which generates optimized
multi-stage Dockerfiles for production deployment.

Fixes #123
```

```
[runtime] Reduce V8 cold start time by 40%

Implemented V8 snapshot caching to pre-compile the core
runtime and extensions. Workers now initialize in ~2ms
instead of ~3-5ms.

Fixes #456
```

---

## 🔍 Areas We Need Help

### High Priority
- [ ] **V8 Snapshot Implementation**: Full `SnapshotCreator` integration
- [ ] **Memory Limits**: Expose `ResourceConstraints` for heap control
- [ ] **WebSocket Support**: Add real-time communication
- [ ] **Database Extensions**: PostgreSQL, MySQL, SQLite drivers
- [ ] **Middleware System**: Express-like middleware support

### Medium Priority
- [ ] **Better Error Messages**: More helpful diagnostics
- [ ] **Performance Monitoring**: Built-in metrics and profiling
- [ ] **Graceful Degradation**: Better handling of worker crashes
- [ ] **Hot Module Replacement**: Faster dev reloads
- [ ] **Static File Serving**: Built-in static asset handler

### Nice to Have
- [ ] **VS Code Extension**: Syntax highlighting, debugging
- [ ] **GitHub Actions**: CI/CD templates
- [ ] **Deployment Guides**: Railway, Fly.io, AWS, GCP
- [ ] **Benchmarking Suite**: Automated performance testing
- [ ] **Video Tutorials**: Getting started, advanced topics

---

## 📚 Resources

### Documentation
- [Main README](./README.md)
- [Changelog](./CHANGELOG.md)
- [Performance Guide](./test-apps/test-js/server/PERFORMANCE.md)
- [Titan Docs](https://titan-docs-ez.vercel.app/docs)

### Community
- [GitHub Issues](https://github.com/ezet-galaxy/titanpl/issues)
- [GitHub Discussions](https://github.com/ezet-galaxy/titanpl/discussions)

### Learning Rust + V8
- [The Rust Book](https://doc.rust-lang.org/book/)
- [V8 Embedder's Guide](https://v8.dev/docs/embed)
- [rusty_v8 Examples](https://github.com/denoland/rusty_v8/tree/main/examples)

---

## 🙏 Thank You!

Every contribution, no matter how small, helps make Titan better for everyone. Whether you're fixing a typo, adding a feature, or improving documentation — **you're building the future of hybrid backend development**. 🚀

**Questions?** Open an issue or start a discussion. We're here to help!

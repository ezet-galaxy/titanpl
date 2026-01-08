# 🪐 Titan Extension: {{name}}

> Elevate Titan Planet with custom JavaScript and high-performance Native Rust logic.

Welcome to your new Titan extension! This template provides everything you need to build, test, and deploy powerful additions to the Titan project.

---

## 🛠 Project Structure

- `index.js`: The JavaScript entry point where you define your extension's API on the global `t` object.
- `titan.json`: Manifest file defining extension metadata and Native module mappings.
- `native/`: Directory for Rust source code.
  - `src/lib.rs`: Your Native function implementations.
  - `Cargo.toml`: Rust package and dependency configuration.
- `jsconfig.json`: Enables full IntelliSense for the Titan Runtime API.

---

## 🚀 Quick Start

### 1. Install Dependencies
Get full type support in your IDE:
```bash
npm install
```

### 2. Build Native Module (Optional)
If your extension uses Rust, compile it to a dynamic library:
```bash
cd native
cargo build --release
cd ..
```

### 3. Test the Extension
Use the Titan SDK to run a local test harness:
```bash
titan run ext
```
*Tip: Visit `http://localhost:3000/test` after starting the runner to see your extension in action!*

---

## 💻 Development Guide

### Writing JavaScript
Extensions interact with the global `t` object. It's best practice to namespace your extension:

```javascript
t.{{name}} = {
    myMethod: (val) => {
        t.log("{{name}}", "Doing something...");
        return val * 2;
    }
};
```

### Writing Native Rust Functions
Native functions should be marked with `#[unsafe(no_mangle)]` and use `extern "C"`:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn multiply(a: f64, b: f64) -> f64 {
    a * b
}
```

### Mapping Native Functions in `titan.json`
Expose your Rust functions to JavaScript by adding them to the `native.functions` section:

```json
"functions": {
    "add": {
        "symbol": "add",
        "parameters": ["f64", "f64"],
        "result": "f64"
    }
}
```

---

## 🧪 Testing with Titan SDK

The `titan run ext` command automates the testing workflow:
1. It builds your native code.
2. It sets up a temporary Titan project environment.
3. It links your extension into `node_modules`.
4. It starts the Titan Runtime at `http://localhost:3000`.

You can modify the test harness or add custom test cases by exploring the generated `.titan_test_run` directory (it is git-ignored).

---

## 📦 Deployment
To use your extension in a Titan project:
1. Publish your extension to npm or link it locally.
2. In your Titan project: `npm install my-extension`.
3. The Titan Runtime will automatically detect and load your extension if it contains a `titan.json`.

---

Happy coding on Titan Planet! 🚀

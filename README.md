***

## TITAN PLANET 🚀  
JavaScript Simplicity. Rust Power.

Titan is a JavaScript-first backend framework that compiles your JS routes and actions into a production-grade **Rust + Axum native server**.

Developers write **zero Rust**, yet deploy a high-performance, safe, fully native backend with excellent DX (developer experience).

Titan = Next.js DX × Rust performance × JavaScript simplicity

---

## ⚙ Requirements

Before using Titan, ensure your system has:

### **1. Rust (latest stable)**
Install from:
https://rust-lang.org/tools/install/

### **2. Node.js (v18+)**
Required for:
- Titan CLI  
- esbuild  
- JS → Rust compilation process  

Check version:
```bash
node -v
npm -v
rustc -V
```

---

## New Features
  To get new features:
  ```bash
  tit update
  ```

## ✨ Features

- Write your backend in **pure JavaScript**
- Compile into a **native Rust HTTP server**
- Titan DSL: `t.get()`, `t.post()`, `t.start()`
- Automatic **route generation**
- Automatic **JS action bundling**
- Fast **Rust Axum runtime**
- JavaScript execution via **Boa engine**
- **Hot Reload Dev Server** (edit → rebuild → restart automatically)
- Production output: **single binary**
- Zero-config deployment

---

## 📦 Installation

Install the Titan CLI globally:

```bash
npm install -g @ezetgalaxy/titan
```

---

## 🚀 Create a New Titan Project

```bash
tit init my-app
cd my-app
tit dev
```

Titan will automatically:

- Create project structure  
- Generate routes from `/app/app.js`  
- Bundle JS actions into `.jsbundle` files  
- Start the **Rust Axum dev server with Hot Reload**  

---

# 📁 Project Structure

```
my-app/
├── app/
│   ├── app.js                 # Titan routes (DSL)
│   └── actions/
│       └── hello.js           # Titan action
│
├── titan/
│   ├── titan.js               # Titan DSL
│   ├── bundle.js              # Bundler (esbuild)
│   └── dev.js                 # Hot reload engine
│
├── server/                    # Rust backend (auto generated)
│   ├── src/
│   ├── actions/               # JS → .jsbundle compiled actions
│   ├── titan/                 # internal runtime files
│   ├── target/                # Cargo build output
│   ├── routes.json
│   ├── action_map.json
│   └── titan-server           # Final Rust binary
│
└── package.json
```

This is the complete Titan architecture:  
**JS input → Rust server output → Native production binary.**

---

# 🧩 Example: Titan Action

**app/actions/hello.js**

```js
function hello(req) {
  return { message: "Hello from Titan!" };
}

globalThis.hello = hello;
```

---

# 🛣 Example: Titan Routes (DSL)

**app/app.js**

```js
import t from "../titan/titan.js";

// POST /hello → hello action
t.post("/hello").action("hello");

// GET / → reply text
t.get("/").reply("Welcome to Titan");

t.start(3000, "Ready to land on Titan Planet 🚀");
```

Titan generates:

- `server/routes.json`
- `server/action_map.json`

Used by the Rust runtime to dispatch requests.

---

# 🔥 Hot Reload Dev Mode

Start development mode:

```bash
tit dev
```

Titan Dev Mode will:

- Regenerate routes on every save  
- Rebundle actions automatically  
- **Kill and restart the Rust server safely**  
- Give full hot reload like modern JS frameworks  

Full DX flow:

```
Save file → auto rebuild → auto restart → updated API
```

Supports:

- Editing `app/app.js`
- Editing `app/actions/*.js`
- Fast rebuilds via esbuild

---

# 🏭 Production Build

```bash
tit build
```

Production output goes into:

```
server/
  titan-server
  routes.json
  action_map.json
  actions/*.jsbundle
```

You deploy **only the server folder**.

---

# ☁ Deploying Titan

After `tit build`, deploy the `server/` folder anywhere:

- Railway  
- Fly.io  
- Docker  
- VPS  
- Render  
- Bare metal  

Start command:

```bash
./titan-server
```

No Node.js needed in production — Titan runs as a pure Rust binary.

---

# 🧠 How Titan Works (Internals)

### 1. JavaScript DSL  
You write backend logic using Titan’s intuitive DSL.

### 2. Bundler  
Titan uses esbuild to compile JS actions into `.jsbundle`.

### 3. Metadata  
`t.start()` writes:

- `routes.json`
- `action_map.json`

### 4. Rust Server  
Axum server:

- Loads `.jsbundle` actions  
- Injects request data  
- Executes JS via Boa  
- Returns JSON response to user  

### 5. Production Output  
Titan produces:

- A **native binary**  
- JS bundles  
- Route maps  
- Entire backend in one folder  

---

# 🎯 Why Titan Exists

Titan exists for developers who want:

- Rust performance  
- JavaScript simplicity  
- Zero Rust learning curve  
- Zero config deployment  
- Modern DX + native speed  

Titan bridges two worlds:

**JavaScript Productivity × Rust Performance**

---

# 📌 Version

**Titan v1 — Stable**

- JS → Rust server compiler  
- Action Engine  
- Axum Runtime  
- Titan DSL  
- Hot Reload Dev Mode  
- Railway/Fly.io Deployment  

---

# 🤝 Contributing

PRs, issues, suggestions, and feature discussions are welcome.

***


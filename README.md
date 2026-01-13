
```
████████╗██╗████████╗ █████╗ ███╗   ██╗   ██████╗  ██████╗ ██████╗  ██████╗
╚══██╔══╝██║╚══██╔══╝██╔══██╗████╗  ██║   ╚════██╗██╔═████╗╚════██╗██╔════╝
   ██║   ██║   ██║   ███████║██╔██╗ ██║    █████╔╝██║██╔██║ █████╔╝███████╗
   ██║   ██║   ██║   ██╔══██║██║╚██╗██║   ██╔═══╝ ████╔╝██║██╔═══╝ ██═══██║
   ██║   ██║   ██║   ██║  ██║██║ ╚████║   ███████╗╚██████╔╝███████╗███████║
   ╚═╝   ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚══════╝ ╚═════╝ ╚══════╝╚══════╝
```

# Notice

💙 **Enjoy development mode `titan dev`**
💟 **Titan Planet docs:** https://titan-docs-ez.vercel.app/docs
🚀 **CLI: `titan` is now the canonical command. `tit` remains supported as an alias.**

---

# TITAN PLANET 🚀 
<a href="https://github.com/ezet-galaxy/titanpl">
<svg xmlns="http://www.w3.org/2000/svg" width="88" height="20" role="img" aria-label="npm: v26.7.4">
    <title>npm: v26.7.4</title>
    <g shape-rendering="crispEdges">
        <rect width="35" height="20" fill="#555" />
        <rect x="35" width="53" height="20" fill="#007ec6" />
    </g>
    <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif"
        text-rendering="geometricPrecision" font-size="110">
        <text x="185" y="140" transform="scale(.1)" fill="#fff" textLength="250">npm</text>
        <text x="605" y="140" transform="scale(.1)" fill="#fff" textLength="430">v26.7.4</text>
    </g>
    <script xmlns="" />
    <script xmlns="" />
</svg>
</a>

**JavaScript Simplicity. Rust Power. Zero Configuration.**

Titan Planet is a JavaScript-first backend framework that embeds JS actions into a Rust + Axum server and ships as a single native binary. Routes are compiled to static metadata; only actions run in the embedded JS runtime. No Node.js. No event loop in production.

You write **zero Rust**.
Titan ships a full backend engine, dev server, bundler, router, action runtime, and Docker deploy pipeline — all powered by Rust under the hood.

Titan = **JavaScript productivity × Rust performance × Zero DevOps**

---

# 🌌 Why Titan?

| Feature                              | Titan | Express/Nest | FastAPI | Bun       |
| ------------------------------------ | ----- | ------------ | ------- | --------- |
| Native binary output                 | ✅ Yes | ❌ No         | ❌ No    | ❌ No      |
| Rust-level performance               | ✅ Yes | ❌ No         | ❌ No    | ❌ No      |
| Pure JavaScript developer experience | ✅ Yes | ✅ Yes        | ❌ No    | ❌ Partial |
| Zero-config Docker deploy            | ✅ Yes | ❌ No         | ❌ No    | ❌ No      |
| Action-based architecture            | ✅ Yes | ❌ No         | ❌ No    | ❌ No      |
| Hot reload dev server                | ✅ Yes | ❌ No         | ❌ No    | ❌ No      |

Titan gives you:

* Native speed
* JS comfort
* Cloud-first deployment
* Full environment variable support
* Built-in HTTP client (`t.fetch`)
* Lightweight serverless-like actions
* Instant hot reload
* Zero configuration
* Single deployable binary

---

# 🚀 Quick Start


# ⚙ Requirements

Install before using Titan:

### 1. Rust (latest stable)

[https://rust-lang.org/tools/install/](https://rust-lang.org/tools/install/)

### 2. Node.js (v18+)

Required for:

* Titan CLI
* esbuild
* JS → Rust compilation pipeline

Verify:

```bash
node -v
npm -v
rustc -V
```

---

### Install Titan CLI

```bash
npm install -g @ezetgalaxy/titan
```

### Create a new project

```bash
titan init my-app
cd my-app
titan dev
```

Titan will:

* Build routes
* Bundle actions
* Start Rust dev server
* Watch file changes
* Trigger instant reload

---

# Update to new version

* At first update the cli 

```bash
npm install -g @ezetgalaxy/titan@latest
```
* Then 

```bash
titan update
```
* ``tit update`` will update and add new features in your Titan project


# ✨ What Titan Can Do (New & Core Features)

Titan now includes a **complete runtime engine** with the following built-in capabilities:

### 🛣 Routing & HTTP

* Static routes (`/`, `/health`)
* Dynamic routes (`/user/:id<number>`)
* Typed route parameters
* Automatic method matching (GET / POST)
* Query parsing (`req.query`)
* Body parsing (`req.body`)
* Zero-config routing metadata generation

### 🧠 Action Runtime

* JavaScript actions executed inside a Rust runtime (v8)
* Automatic action discovery and execution
* No `globalThis` required anymore
* Safe handling of `undefined` returns
* JSON serialization guardrails
* Action-scoped execution context

### 🔌 Runtime APIs (`t`)

* `t.fetch(...)` — built-in Rust-powered HTTP client
* `t.log(...)` — sandboxed, action-scoped logging
* Environment variable access (`process.env`)
* No access to raw Node.js APIs (safe by default)

### 🧾 Request Object (`req`)

Each action receives a normalized request object:

```json
{
  "method": "GET",
  "path": "/user/90",
  "params": { "id": "90" },
  "query": {},
  "body": null,

  "headers": {
    "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
    "accept-encoding": "gzip, deflate, br, zstd",
    "accept-language": "en-US,en;q=0.9",
    "cache-control": "max-age=0",
    "connection": "keep-alive",
    "cookie": "",
    "host": "localhost:3000",
    "sec-ch-ua": "\"Google Chrome\";v=\"143\", \"Chromium\";v=\"143\", \"Not A(Brand\";v=\"24\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"Windows\"",
    "sec-fetch-dest": "document",
    "sec-fetch-mode": "navigate",
    "sec-fetch-site": "none",
    "sec-fetch-user": "?1",
    "upgrade-insecure-requests": "1",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36"
  },
}
```

This object is:

* Stable
* Predictable
* Serializable
* Identical across dev & production

---

### 🔥 Developer Experience

* Hot reload dev server (`titan dev`)
* Automatic rebundling of actions
* Automatic Rust server restart
* Colored request logs
* Per-route timing metrics
* Action-aware logs

Example runtime log:

```
[Titan] GET /user/90 → getUser (dynamic) in 0.42ms
[Titan] log(getUser): Fetching user 90
```

---

### 🧨 Error Handling & Diagnostics

* JavaScript runtime errors captured safely
* Action-aware error reporting
* Line & column hints from runtime
* Red-colored error logs
* No server crashes on user mistakes
* Safe fallback for `undefined` returns

---

### ⚙ Build & Deployment

* Native Rust binary output
* Zero-config Dockerfile generation
* Multi-stage optimized Docker builds
* Works on:

  * Railway
  * Fly.io
  * Render
  * VPS
  * Kubernetes
* No Node.js required in production

---

---

### 🧩 Titan Extensions & Test Harness

Titan Planet isn't just a framework; it's an extensible platform. You can build custom extensions to add new native capabilities to the `t` runtime.

*   **`titan create ext <name>`**: Scaffold a new Titan Extension template (supports JS and Native Rust modules).
*   **`titan run ext`**: Launch the **Titan Test Harness**. This provides a "lite" Titan runtime environment that automatically:
    *   Builds your native Rust code.
    *   Links your extension to a temporary project.
    *   Generates a test suite to verify your extension's methods.
    *   Starts a live server to test extension logic in a real-world scenario.

---

### 🧱 Architecture Guarantees

# 🧩 Example Action (Updated – No `globalThis` Needed)

```js
export function getUser(req) {
  t.log("User id:", req.params.id);

  return {
    id: Number(req.params.id),
    method: req.method
  };
}
```

That’s it.
No exports wiring. No globals. No boilerplate.

---

# 📦 Version

**Titan v26 — Stable**

* Production-ready runtime
* Safe JS execution
* Native Rust performance
* Designed for cloud & AI workloads

---

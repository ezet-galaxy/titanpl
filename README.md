
***

## TITAN 🚀
JavaScript Simplicity. Rust Power.

Titan is a JavaScript-first backend framework that compiles your JS routes and actions into a production-grade Rust + Axum server.

Developers write zero Rust, yet deploy a native Rust backend with extreme performance, safety, and scalability.

Titan turns your JavaScript backend into:

- A Rust Axum HTTP server  
- A JS action execution runtime (via Boa)  
- A standalone binary ready for Railway, Fly.io, VPS, Docker  
- A fully portable server with no Node.js required in production  

Titan = Next.js DX × Rust performance × JS developer simplicity

### Features

- Write backend logic in JavaScript  
- Compile into native Rust backend  
- Titan DSL (t.post(), t.start())  
- Automatic route generation  
- Automatic JS action bundling  
- Rust Axum server runtime  
- JavaScript execution via Boa (sandboxed)  
- Hot-reload dev server   // development in progress
- Production binary output  
- Zero-config deployment

### Installation

Install the Titan CLI globally:

```bash
npm install -g titan-cli
```

### Create a New Titan Project

```bash
tit init my-app
cd my-app
tit dev
```

This will:

- Generate Titan project structure  
- Build routes from /app/app.js  
- Bundle JS actions into [.jsbundle] files  
- Start the Rust Axum development server with hot reload

### Project Structure

```
my-app/
├── app/
│   ├── app.js
│   └── actions/
│       └── hello.js
│
├── titan/
│   |── titan.js
|   |__ bundle.js
│
├── cli/
│   └── bundle.js
│
├── server/            ← Rust backend ([translate:auto generated])
│   ├── src/
│   ├── actions/
│   ├── titan/
│   ├── target/
│   ├── routes.json
│   ├── action_map.json
│   └── titan-server   ← final binary
│
└── package.json
```

### Example: Titan Action

**app/actions/hello.js**

```js
function hello(req) {
  return { message: "Hello from Titan!" };
}

globalThis.hello = hello
```

This registers a global function _hello_ for the Rust runtime.

### Example: Titan Routes

**app/app.js**

```js
import t from "../titan/titan.js";


// POST /hello → hello action
t.post("/hello").action("hello");

// GET / → reply text
t.get("/").reply("Welcome to Titan");

t.start(3000, "Titan is running!");
```

Titan generates routing metadata:

- server/routes.json  
- server/action_map.json  

These are then used by the Rust server.

### Development Mode

```bash
tit dev
```

Titan will:

- Generate route definitions  
- Bundle JS into .jsbundle files  
- Start Axum Rust server with live reload

### Production Build

```bash
tit build
```

This produces the final deployment-ready output:

```
server/
  titan-server          ← release binary
  routes.json
  action_map.json
  actions/*.jsbundle
  titan/titan.jsbundle
```

Everything required for production is inside the server/ folder.

### Deploying Titan

You deploy only the /server folder.

Example (Railway):

Build locally:

```bash
tit build
```

Upload the /server folder

Set start command:

```bash
./titan-server
```

No Node.js needed in production. Titan servers run as pure Rust native binaries.

### How Titan Works Internally

1. JavaScript DSL

You write server logic using the Titan DSL:

- t.get()  
- t.post()  
- t.start()

2. Bundler

Titan bundles actions using esbuild into .jsbundle.


4. Rust Server

The Rust Axum server:

- Loads .jsbundle files  
- Injects request data  
- Executes JS functions via Boa  
- Returns Rust → JSON → client

5. Production Output

Titan outputs:

- Native Rust binary  
- JS bundles  
- Route maps

### Why Titan Exists

Titan targets JS developers who want:

- Rust backend performance  
- Without needing Rust knowledge  
- With full JS developer experience  
- And deployment as easy as Node

Titan bridges two worlds:

JavaScript flexibility + Rust performance

### Version

Titan v1 (Current)

- JS → Rust server compiler  
- JavaScript Action Engine  
- Axum runtime  
- Titan DSL  
- Hot reload  
- Railway deployment


### Contributing

PRs, issues, and discussions are welcome.

***
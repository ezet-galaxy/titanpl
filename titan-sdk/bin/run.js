#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Helper for colors
const cyan = (t) => `\x1b[36m${t}\x1b[0m`;
const green = (t) => `\x1b[32m${t}\x1b[0m`;
const red = (t) => `\x1b[31m${t}\x1b[0m`;
const yellow = (t) => `\x1b[33m${t}\x1b[0m`;

function run() {
    console.log(cyan("Titan SDK: Test Runner"));

    // 1. Validate we are in an extension directory
    const cwd = process.cwd();
    const manifestPath = path.join(cwd, "titan.json");
    if (!fs.existsSync(manifestPath)) {
        console.log(red("Error: titan.json not found. Run this command inside your extension folder."));
        process.exit(1);
    }

    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    const name = manifest.name;
    console.log(green(`Extension: ${name}`));

    // 2. Build Native Logic (if properly set up)
    const nativeDir = path.join(cwd, "native");
    if (fs.existsSync(nativeDir) && fs.existsSync(path.join(nativeDir, "Cargo.toml"))) {
        console.log(cyan("Building native Rust module..."));
        try {
            execSync("cargo build --release", { cwd: nativeDir, stdio: "inherit" });
        } catch (e) {
            console.log(red("Failed to build native module."));
            process.exit(1);
        }
    }

    // 3. Create a Test Harness (Mini Titan Project)
    const runDir = path.join(cwd, ".titan_test_run");
    if (fs.existsSync(runDir)) {
        fs.rmSync(runDir, { recursive: true, force: true });
    }
    fs.mkdirSync(runDir);

    // Create app structure
    const appDir = path.join(runDir, "app");
    fs.mkdirSync(appDir);

    // Create actions folder (required by Titan build)
    const actionsDir = path.join(appDir, "actions");
    fs.mkdirSync(actionsDir);

    // Copy titan/ and server/ from templates
    const repoRoot = path.resolve(__dirname, "..", "..");
    const templatesDir = path.join(repoRoot, "templates");

    const titanSrc = path.join(templatesDir, "titan");
    const titanDest = path.join(runDir, "titan");
    if (fs.existsSync(titanSrc)) {
        fs.cpSync(titanSrc, titanDest, { recursive: true });
    }

    const serverSrc = path.join(templatesDir, "server");
    const serverDest = path.join(runDir, "server");
    if (fs.existsSync(serverSrc)) {
        fs.cpSync(serverSrc, serverDest, { recursive: true });
    }

    // Create package.json for the test harness
    const pkgJson = {
        "type": "module"
    };
    fs.writeFileSync(path.join(runDir, "package.json"), JSON.stringify(pkgJson, null, 2));

    // Create 'node_modules' to link the extension
    const nmDir = path.join(runDir, "node_modules");
    fs.mkdirSync(nmDir);

    // Link current extension to node_modules/NAME
    // Use junction for Windows compat without admin rights
    const extDest = path.join(nmDir, name);
    try {
        fs.symlinkSync(cwd, extDest, "junction");
    } catch (e) {
        // Fallback to copy if link fails
        console.log(yellow("Linking failed, copying extension files..."));
        fs.cpSync(cwd, extDest, { recursive: true });
    }

    // Create a test action in app/actions/test.js
    const testAction = `export const test = (req) => {
    const ext = t["${name}"];
    
    const results = {
        extension: "${name}",
        loaded: !!ext,
        methods: ext ? Object.keys(ext) : [],
        timestamp: new Date().toISOString()
    };
    
    if (ext && ext.hello) {
        try {
            results.hello_test = ext.hello("World");
        } catch(e) {
            results.hello_error = String(e);
        }
    }
    
    if (ext && ext.calc) {
        try {
            results.calc_test = ext.calc(15, 25);
        } catch(e) {
            results.calc_error = String(e);
        }
    }
    
    return results;
};
`;

    fs.writeFileSync(path.join(actionsDir, "test.js"), testAction);

    // Create a simple test script in app/app.js
    // This script will be executed by Titan
    const testScript = `import t from "../titan/titan.js";

// Extension test harness for: ${name}
const ext = t["${name}"];

console.log("---------------------------------------------------");
console.log("Testing Extension: ${name}");
console.log("---------------------------------------------------");

if (!ext) {
    console.log("ERROR: Extension '${name}' not found in global 't'.");
    console.log("Make sure your extension's package.json has 'type': 'commonjs'");
} else {
    console.log("✓ Extension loaded successfully!");
    console.log("✓ Available methods:", Object.keys(ext).join(", "));
    
    // Try 'hello' if it exists
    if (typeof ext.hello === 'function') {
        console.log("\\nTesting ext.hello('Titan')...");
        try {
           const res = ext.hello("Titan");
           console.log("✓ Result:", res);
        } catch(e) {
           console.log("✗ Error:", e.message);
        }
    }

    // Try 'calc' if it exists
    if (typeof ext.calc === 'function') {
        console.log("\\nTesting ext.calc(10, 20)...");
        try {
            const res = ext.calc(10, 20);
            console.log("✓ Result:", res);
        } catch(e) {
            console.log("✗ Error:", e.message);
        }
    }
}

console.log("---------------------------------------------------");
console.log("✓ Test complete!");
console.log("\\n📍 Routes:");
console.log("  GET  http://localhost:3000/      → Test harness info");
console.log("  GET  http://localhost:3000/test  → Extension test results (JSON)");
console.log("---------------------------------------------------\\n");

// Create routes
t.get("/test").action("test");
t.get("/").reply("🚀 Extension Test Harness for ${name}\\n\\nVisit /test to see extension test results");

t.start(3000, "Titan Extension Test Running!");
`;

    fs.writeFileSync(path.join(appDir, "app.js"), testScript);

    // Build the app (bundle actions)
    console.log(cyan("Building test app..."));
    try {
        execSync("node app/app.js --build", { cwd: runDir, stdio: "inherit" });
    } catch (e) {
        console.log(red("Failed to build test app."));
        console.log(yellow("This is expected if your extension has errors."));
    }

    // 4. Run Titan Server using cargo run (like dev mode)
    console.log(cyan("Starting Titan Runtime..."));

    const serverDir = path.join(runDir, "server");

    try {
        execSync("cargo run", { cwd: serverDir, stdio: "inherit" });
    } catch (e) {
        console.log(red("Runtime exited."));
    }
}

run();

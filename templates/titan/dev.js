import chokidar from "chokidar";
import { spawn, execSync } from "child_process";
import path from "path";
import { fileURLToPath } from "url";
import fs from "fs";
import { bundle } from "./bundle.js";

// Required for __dirname in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let serverProcess = null;
let isKilling = false;

async function killServer() {
    if (!serverProcess) return;

    isKilling = true;
    const pid = serverProcess.pid;
    const killPromise = new Promise((resolve) => {
        if (serverProcess.exitCode !== null) return resolve();
        serverProcess.once("close", resolve);
    });

    if (process.platform === "win32") {
        try {
            execSync(`taskkill /pid ${pid} /f /t`, { stdio: 'ignore' });
        } catch (e) {
            // Ignore errors if process is already dead
        }
    } else {
        serverProcess.kill();
    }

    try {
        await killPromise;
    } catch (e) { }
    serverProcess = null;
    isKilling = false;
}

async function startRustServer(retryCount = 0) {
    // If we are retrying, give it more time (2s), otherwise standard 1s (increased from 500ms)
    const waitTime = retryCount > 0 ? 2000 : 1000;

    // Ensure any previous instance is killed
    await killServer();

    // Give the OS a moment to release file locks on the binary
    await new Promise(r => setTimeout(r, waitTime));

    const serverPath = path.join(process.cwd(), "server");
    const startTime = Date.now();

    if (retryCount > 0) {
        console.log(`\x1b[33m[Titan] Retrying Rust server (Attempt ${retryCount})...\x1b[0m`);
    }

    // Windows often has file locking issues during concurrent linking/metadata generation
    // We force 1 job and disable incremental compilation to be safe.
    serverProcess = spawn("cargo", ["run", "--jobs", "1"], {
        cwd: serverPath,
        stdio: "inherit",
        shell: true,
        env: { ...process.env, CARGO_INCREMENTAL: "0" }
    });

    serverProcess.on("close", async (code) => {
        if (isKilling) return;

        console.log(`[Titan] Rust server exited: ${code}`);

        // If exited with error and it was a short run (< 10s), likely a start-up error/lock
        // Retry up to 3 times
        const runTime = Date.now() - startTime;
        if (code !== 0 && code !== null && runTime < 10000 && retryCount < 3) {
            console.log(`\x1b[31m[Titan] Server crash detected (possibly file lock). Retrying automatically...\x1b[0m`);
            await startRustServer(retryCount + 1);
        }
    });
}

async function rebuild() {
    console.log("[Titan] Regenerating routes.json & action_map.json...");
    execSync("node app/app.js", { stdio: "inherit" });

    console.log("[Titan] Bundling JS actions...");
    await bundle();
}

async function startDev() {
    console.log("[Titan] Dev mode starting...");

    if (fs.existsSync(path.join(process.cwd(), ".env"))) {
        console.log("\x1b[33m[Titan] Env Configured\x1b[0m");
    }

    // FIRST BUILD
    try {
        await rebuild();
        await startRustServer();
    } catch (e) {
        console.log("\x1b[31m[Titan] Initial build failed. Waiting for changes...\x1b[0m");
    }

    const watcher = chokidar.watch(["app", ".env"], {
        ignoreInitial: true
    });

    let timer = null;

    watcher.on("all", async (event, file) => {
        if (timer) clearTimeout(timer);

        timer = setTimeout(async () => {
            if (file.includes(".env")) {
                console.log("\x1b[33m[Titan] Env Refreshed\x1b[0m");
            } else {
                console.log(`[Titan] Change detected: ${file}`);
            }

            try {
                await rebuild();
                console.log("[Titan] Restarting Rust server...");
                await startRustServer();
            } catch (e) {
                console.log("\x1b[31m[Titan] Build failed -- waiting for changes...\x1b[0m");
            }

        }, 500);
    });
}

// Handle graceful exit to release file locks
async function handleExit() {
    console.log("\n[Titan] Stopping server...");
    await killServer();
    process.exit(0);
}

process.on("SIGINT", handleExit);
process.on("SIGTERM", handleExit);

startDev();

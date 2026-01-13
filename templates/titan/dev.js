import chokidar from "chokidar";
import { spawn, execSync } from "child_process";
import path from "path";
import { fileURLToPath } from "url";
import fs from "fs";
import { bundle } from "./bundle.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let serverProcess = null;

/**
 * Detect if project uses TypeScript or JavaScript
 */
function getAppEntry() {
  const root = process.cwd();
  const tsEntry = path.join(root, "app", "app.ts");
  const jsEntry = path.join(root, "app", "app.js");

  if (fs.existsSync(tsEntry)) return { path: tsEntry, isTS: true };
  if (fs.existsSync(jsEntry)) return { path: jsEntry, isTS: false };

  return null;
}

/**
 * Compile TypeScript app.ts to JavaScript using esbuild
 */
async function compileAndRunAppEntry() {
  const entry = getAppEntry();

  if (!entry) {
    throw new Error("[Titan] No app.ts or app.js found in app/");
  }

  const root = process.cwd();

  if (entry.isTS) {
    console.log("[Titan] Compiling app.ts with esbuild...");

    const esbuild = await import("esbuild");
    const titanDir = path.join(root, ".titan");
    const outFile = path.join(titanDir, "app.compiled.mjs");

    // Clean and recreate .titan directory to avoid cache issues
    if (fs.existsSync(titanDir)) {
      fs.rmSync(titanDir, { recursive: true, force: true });
    }
    fs.mkdirSync(titanDir, { recursive: true });

    // Compile TS to JS WITHOUT bundling
    await esbuild.build({
      entryPoints: [entry.path],
      outfile: outFile,
      format: "esm",
      platform: "node",
      target: "node18",
      bundle: false,
      loader: { ".ts": "ts" },
      tsconfigRaw: {
        compilerOptions: {
          experimentalDecorators: true,
          useDefineForClassFields: true,
        },
      },
    });

    // Read and fix imports
    let compiled = fs.readFileSync(outFile, "utf8");
    
    // Convert relative titan import to absolute path
    const titanPath = path.join(root, "titan", "titan.js").replace(/\\/g, "/");
    
    // Replace the import statement - handle with or without semicolon
    compiled = compiled.replace(
      /import\s+(\w+)\s+from\s+["']\.\.\/titan\/titan\.js["'];?/g,
      `import $1 from "${titanPath}";`
    );

    fs.writeFileSync(outFile, compiled);
    
    // Debug: show first 3 lines of compiled output
    console.log("[Titan] Compiled output preview:");
    const lines = compiled.split("\n").slice(0, 3);
    lines.forEach((line, i) => console.log(`  ${i + 1}: ${line}`));

    // Execute
    execSync(`node "${outFile}"`, { stdio: "inherit", cwd: root });
  } else {
    execSync(`node "${entry.path}"`, { stdio: "inherit", cwd: root });
  }
}

async function killServer() {
  if (!serverProcess) return;

  const pid = serverProcess.pid;
  const killPromise = new Promise((resolve) => {
    if (serverProcess.exitCode !== null) return resolve();
    serverProcess.once("close", resolve);
  });

  if (process.platform === "win32") {
    try {
      execSync(`taskkill /pid ${pid} /f /t`, { stdio: "ignore" });
    } catch (e) {
      // Ignore errors if process is already dead
    }
  } else {
    serverProcess.kill();
  }

  try {
    await killPromise;
  } catch (e) {}
  serverProcess = null;
}

async function startRustServer() {
  await killServer();

  // Give the OS a moment to release file locks on the binary
  await new Promise((r) => setTimeout(r, 500));

  const serverPath = path.join(process.cwd(), "server");

  serverProcess = spawn("cargo", ["run"], {
    cwd: serverPath,
    stdio: "inherit",
    shell: true,
  });

  serverProcess.on("close", (code) => {
    if (code !== null && code !== 0 && code !== 1) {
      // 1 is often just 'terminated' on windows if forced, but also error.
    }
    console.log(`[Titan] Rust server exited: ${code}`);
  });
}

async function rebuild() {
  console.log("[Titan] Regenerating routes.json & action_map.json...");

  // Compile and run app entry (TS or JS)
  await compileAndRunAppEntry();

  console.log("[Titan] Bundling actions...");
  await bundle();
}

async function startDev() {
  console.log("[Titan] Dev mode starting...");

  const entry = getAppEntry();
  if (entry) {
    console.log(
      `[Titan] Detected ${entry.isTS ? "TypeScript" : "JavaScript"} project`
    );
  }

  if (fs.existsSync(path.join(process.cwd(), ".env"))) {
    console.log("\x1b[33m[Titan] Env Configured\x1b[0m");
  }

  // FIRST BUILD
  try {
    await rebuild();
    await startRustServer();
  } catch (e) {
    console.log(
      "\x1b[31m[Titan] Initial build failed. Waiting for changes...\x1b[0m"
    );
    console.error(e.message);
  }

  // Watch for changes - include .ts files
  const watcher = chokidar.watch(["app/**/*.{js,ts}", "titan/**/*.js", ".env"], {
    ignoreInitial: true,
    ignored: ["**/*.d.ts", "**/node_modules/**"],
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
        console.log(
          "\x1b[31m[Titan] Build failed -- waiting for changes...\x1b[0m"
        );
        console.error(e.message);
      }
    }, 200);
  });
}

startDev();
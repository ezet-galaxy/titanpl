#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { execSync, spawn } from "child_process";
import { fileURLToPath } from "url";

// __dirname in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Colors
const cyan = (t) => `\x1b[36m${t}\x1b[0m`;
const green = (t) => `\x1b[32m${t}\x1b[0m`;
const yellow = (t) => `\x1b[33m${t}\x1b[0m`;
const red = (t) => `\x1b[31m${t}\x1b[0m`;
const bold = (t) => `\x1b[1m${t}\x1b[0m`;

const args = process.argv.slice(2);
const cmd = args[0];

// COPY TEMPLATES
function copyDir(src, dest) {
    fs.mkdirSync(dest, { recursive: true });

    for (const file of fs.readdirSync(src)) {
        const srcPath = path.join(src, file);
        const destPath = path.join(dest, file);

        if (fs.lstatSync(srcPath).isDirectory()) {
            copyDir(srcPath, destPath);
        } else {
            fs.copyFileSync(srcPath, destPath);
        }
    }
}

// HELP
function help() {
    console.log(`
${bold(cyan("Titan CLI"))}

${green("tit init <project>")}   Create new Titan project
${green("tit dev")}              Dev mode (hot reload)
${green("tit build")}            Build production Rust server
${green("tit start")}            Start production binary
${green("tit update")}           Update to latest version
`);
}

// INIT PROJECT
function initProject(name) {
    if (!name) return console.log(red("Usage: tit init <project>"));

    const target = path.join(process.cwd(), name);
    const templateDir = path.join(__dirname, "templates");

    if (fs.existsSync(target)) {
        console.log(yellow(`Folder already exists: ${target}`));
        return;
    }

    console.log(cyan(`Creating Titan project → ${target}`));

    copyDir(templateDir, target);
    copyDir(templateDir, target);

    [".gitignore", ".dockerignore", "Dockerfile"].forEach((file) => {
        const src = path.join(templateDir, file);
        const dest = path.join(target, file);
        if (fs.existsSync(src)) fs.copyFileSync(src, dest);
    });


    console.log(green("✔ Titan project created!"));
    console.log(cyan("Installing dependencies..."));

    execSync(`npm install esbuild --silent`, {
        cwd: target,
        stdio: "inherit"
    });

    console.log(green("✔ Dependencies installed"));
    console.log(`
Next steps:
  cd ${name}
  tit dev
`);
}

// BUNDLE
function runBundler() {
    const bundler = path.join(process.cwd(), "titan", "bundle.js");

    if (fs.existsSync(bundler)) {
        execSync(`node ${bundler}`, { stdio: "inherit" });
    } else {
        console.log(yellow("Warning: titan/bundle.js missing."));
    }
}

// ------------------------------------------
// FULL HOT RELOAD DEV SERVER
// ------------------------------------------

async function devServer() {
    console.log(cyan("Titan Dev Mode — Hot Reload Enabled"));

    let rustProcess = null;

    function startRust() {
        return new Promise((resolve) => {
            // if server already running → kill it
            if (rustProcess) {
                console.log("[Titan] Killing old Rust server...");

                if (process.platform === "win32") {
                    const killer = spawn("taskkill", ["/PID", rustProcess.pid, "/T", "/F"], {
                        stdio: "ignore",
                        shell: true,
                    });

                    killer.on("exit", () => {
                        rustProcess = launchRust(resolve);
                    });
                } else {
                    rustProcess.kill();
                    rustProcess.on("close", () => {
                        rustProcess = launchRust(resolve);
                    });
                }
            } else {
                rustProcess = launchRust(resolve);
            }
        });
    }

    function launchRust(done) {
        const processHandle = spawn("cargo", ["run"], {
            cwd: path.join(process.cwd(), "server"),
            stdio: "inherit",
            shell: true,
        });

        processHandle.on("spawn", () => {
            setTimeout(done, 200); // wait for OS to release port
        });

        processHandle.on("close", (code) => {
            console.log(`[Titan] Rust server exited: ${code}`);
        });

        return processHandle;
    }


    function rebuild() {
        console.log(cyan("Titan: Regenerating routes..."));
        execSync("node app/app.js", { stdio: "inherit" });

        console.log(cyan("Titan: Bundling actions..."));
        runBundler();
    }

    // First build
    rebuild();
    startRust();

    // WATCHER
    const chokidar = (await import("chokidar")).default;

    const watcher = chokidar.watch("app", { ignoreInitial: true });

    let timer = null;

    watcher.on("all", (event, file) => {
        if (timer) clearTimeout(timer);

        timer = setTimeout(() => {
            console.log(yellow(`Change detected → ${file}`));

            rebuild();
            startRust();

        }, 250);
    });
}


// BUILD RELEASE — PRODUCTION READY
function buildProd() {
    console.log(cyan("Titan: Building production output..."));

    const projectRoot = process.cwd();

    const appJs = path.join(projectRoot, "app", "app.js");
    const bundler = path.join(projectRoot, "titan", "bundle.js");
    const serverDir = path.join(projectRoot, "server");

    // 1) Ensure app/app.js exists
    if (!fs.existsSync(appJs)) {
        console.log(red("ERROR: app/app.js not found. Cannot build Titan project."));
        process.exit(1);
    }

    // 2) Ensure bundler exists
    if (!fs.existsSync(bundler)) {
        console.log(red("ERROR: titan/bundle.js not found. Cannot bundle actions."));
        process.exit(1);
    }

    // 3) Generate routes.json + action_map.json
    console.log(cyan("→ Generating Titan metadata (routes + action_map)..."));
    try {
        execSync(`node app/app.js --build`, { stdio: "inherit" });
    } catch (err) {
        console.log(red("Failed to generate metadata via app/app.js"));
        process.exit(1);
    }

    // 4) Bundle JS actions → .jsbundle files
    console.log(cyan("→ Bundling Titan actions..."));
    try {
        execSync(`node titan/bundle.js`, { stdio: "inherit" });
    } catch (err) {
        console.log(red("Bundler failed. Check titan/bundle.js for errors."));
        process.exit(1);
    }

    // 5) Ensure server/actions folder exists
    const actionsOut = path.join(serverDir, "actions");
    if (!fs.existsSync(actionsOut)) {
        fs.mkdirSync(actionsOut, { recursive: true });
    }

    // 6) Copy generated bundles into server/actions
    const builtActions = path.join(projectRoot, "titan", "actions");
    if (fs.existsSync(builtActions)) {
        for (const file of fs.readdirSync(builtActions)) {
            if (file.endsWith(".jsbundle")) {
                fs.copyFileSync(
                    path.join(builtActions, file),
                    path.join(actionsOut, file)
                );
            }
        }
    }

    console.log(green("✔ Bundles copied to server/actions"));

    // 7) Build Rust binary
    console.log(cyan("→ Building Rust release binary..."));
    try {
        execSync(`cargo build --release`, {
            cwd: serverDir,
            stdio: "inherit",
        });
    } catch (err) {
        console.log(red("Rust build failed."));
        process.exit(1);
    }

    console.log(green("✔ Titan production build complete!"));
    console.log(green("✔ Rust binary ready at server/target/release/"));
}


// START PRODUCTION
function startProd() {
    const isWindows = process.platform === "win32";
    const binaryName = isWindows ? "titan-server.exe" : "titan-server";

    const exe = path.join(
        process.cwd(),
        "server",
        "target",
        "release",
        binaryName
    );

    execSync(`"${exe}"`, { stdio: "inherit" });
}

// ------------------------------------------
// TITAN UPDATE — Upgrade titan/ runtime
// ------------------------------------------
function updateTitan() {
    const projectRoot = process.cwd();
    const projectTitan = path.join(projectRoot, "titan");

    const cliTemplatesRoot = path.join(__dirname, "templates");
    const cliTitan = path.join(cliTemplatesRoot, "titan");

    if (!fs.existsSync(projectTitan)) {
        console.log(red("No titan/ folder found in this project."));
        console.log(yellow("Make sure you are inside a Titan project."));
        return;
    }


    //
    // 2. Replace titan/ runtime folder
    //
    fs.rmSync(projectTitan, { recursive: true, force: true });
    console.log(green("✔ Old titan/ runtime removed"));

    copyDir(cliTitan, projectTitan);
    console.log(green("✔ titan/ runtime updated"));

    //
    // 3. Update server/Cargo.toml
    //
    const srcToml = path.join(cliTemplatesRoot, "server", "Cargo.toml");
    const destToml = path.join(projectRoot, "server", "Cargo.toml");
    if (fs.existsSync(srcToml)) {
        fs.copyFileSync(srcToml, destToml);
        console.log(green("✔ Updated server/Cargo.toml"));
    }

    //
    // 4. Update ONLY server/src/main.rs
    //
    const srcMain = path.join(cliTemplatesRoot, "server", "src", "main.rs");
    const destMain = path.join(projectRoot, "server", "src", "main.rs");
    if (fs.existsSync(srcMain)) {
        fs.copyFileSync(srcMain, destMain);
        console.log(green("✔ Updated server/src/main.rs"));
    }

    //
    // 5. Update root-level config files
    //
    [".gitignore", ".dockerignore", "Dockerfile"].forEach((file) => {
        const src = path.join(cliTemplatesRoot, file);
        const dest = path.join(projectRoot, file);

        if (fs.existsSync(src)) {
            fs.copyFileSync(src, dest);
            console.log(green(`✔ Updated ${file}`));
        }
    });

    console.log(cyan("✔ Titan forced update complete"));
}






// ROUTER
switch (cmd) {
    case "init":
        initProject(args[1]);
        break;

    case "dev":
        devServer();
        break;

    case "build":
        buildProd();
        break;

    case "start":
        startProd();
        break;

    case "update":
        updateTitan();
        break;

    default:
        help();
}


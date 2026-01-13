#!/usr/bin/env node
import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

import os from "os";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const TITANPL_DIR = path.join(__dirname, "..");
const TMP_DIR = os.tmpdir();
const TEST_DIR = path.join(TMP_DIR, "test-project");

console.log("🔄 Testing Titan CLI changes...\n");

// 1. npm link
console.log("→ Linking titanpl...");
execSync("npm link", { cwd: TITANPL_DIR, stdio: "inherit" });

// 2. Remove old test project
if (fs.existsSync(TEST_DIR)) {
    console.log("→ Removing old test-project...");
    fs.rmSync(TEST_DIR, { recursive: true, force: true });
}

// 3. Create new project
console.log(`→ Creating test-project in ${TMP_DIR}...`);
// On Windows, 'titan' might need to be called as 'titan.cmd' if not fully resolved by shell
const titanCmd = process.platform === 'win32' ? 'titan.cmd' : 'titan';
try {
    execSync(`${titanCmd} init test-project`, { cwd: TMP_DIR, stdio: "inherit" });
} catch (e) {
    // If titan.cmd fails or isn't found, try just 'titan' or npx
    console.log("→ Retry with npx...");
    execSync(`npx titan init test-project`, { cwd: TMP_DIR, stdio: "inherit" });
}

// 4. Show results
console.log("\n📁 Contents of test-project:");
if (fs.existsSync(TEST_DIR)) {
    const files = fs.readdirSync(TEST_DIR);
    files.forEach(file => {
        const stats = fs.statSync(path.join(TEST_DIR, file));
        console.log(`${stats.isDirectory() ? 'd' : '-'} ${file}`);
    });
} else {
    console.error("❌ Test project directory was not created.");
}
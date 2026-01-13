import fs from "fs";
import path from "path";
import esbuild from "esbuild";

const root = process.cwd();
const actionsDir = path.join(root, "app", "actions");
const outDir = path.join(root, "server", "actions");

/**
 * Bundle all actions (JS and TS) using esbuild
 */
export async function bundle() {
  console.log("[Titan] Bundling actions...");

  fs.mkdirSync(outDir, { recursive: true });

  // Clean old bundles
  for (const file of fs.readdirSync(outDir)) {
    if (file.endsWith(".jsbundle")) {
      fs.unlinkSync(path.join(outDir, file));
    }
  }

  // Support both .js and .ts files
  const files = fs
    .readdirSync(actionsDir)
    .filter((f) => /\.(js|ts)$/.test(f) && !f.endsWith(".d.ts"));

  if (files.length === 0) {
    console.log("[Titan] No actions found to bundle.");
    return;
  }

  for (const file of files) {
    const actionName = path.basename(file, path.extname(file));
    const entry = path.join(actionsDir, file);
    const outfile = path.join(outDir, actionName + ".jsbundle");

    console.log(`[Titan] Bundling ${file} → ${actionName}.jsbundle`);

    await esbuild.build({
      entryPoints: [entry],
      outfile,
      bundle: true,
      format: "iife",
      globalName: "__titan_exports",
      platform: "neutral",
      target: "es2020",

      // TypeScript support
      loader: {
        ".ts": "ts",
        ".js": "js",
      },

      // Strip types, no need for declaration files
      tsconfigRaw: {
        compilerOptions: {
          experimentalDecorators: true,
          useDefineForClassFields: true,
        },
      },

      banner: {
        js: "const defineAction = (fn) => fn;",
      },

      footer: {
        js: `
(function () {
  const fn =
    __titan_exports["${actionName}"] ||
    __titan_exports.default;

  if (typeof fn !== "function") {
    throw new Error("[Titan] Action '${actionName}' not found or not a function");
  }

  globalThis["${actionName}"] = fn;
})();
`,
      },
    });
  }

  console.log("[Titan] Bundling finished.");
}

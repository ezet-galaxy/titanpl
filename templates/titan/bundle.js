import fs from "fs";
import path from "path";
import esbuild from "esbuild";

const root = process.cwd();
const actionsDir = path.join(root, "app", "actions");
const outDir = path.join(root, "server", "actions");

fs.mkdirSync(outDir, { recursive: true });

const files = fs.readdirSync(actionsDir).filter(f => f.endsWith(".js"));

(async () => {
  for (const file of files) {
    const entry = path.join(actionsDir, file);
    const outfile = path.join(outDir, file + "bundle");

    console.log(`Bundling ${entry} → ${outfile}`);

    await esbuild.build({
      entryPoints: [entry],
      bundle: false,
      format: "cjs",
      platform: "neutral",
      outfile,
      minify: false,
    });
  }

  console.log("Bundling complete.");
})();

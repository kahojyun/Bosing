import * as esbuild from "esbuild";
import * as fs from "fs";

// clean dist folder
fs.rmSync("./wwwroot/dist", { recursive: true, force: true });
console.log("dist folder cleaned");

// copy static files
fs.copyFileSync(
  "./node_modules/scichart/_wasm/scichart2d.data",
  "./wwwroot/scichart2d.data",
);
fs.copyFileSync(
  "./node_modules/scichart/_wasm/scichart2d.wasm",
  "./wwwroot/scichart2d.wasm",
);

const sharedConfig = {
  entryPoints: ["./**/*.razor.ts"],
  bundle: true,
  splitting: true,
  format: "esm",
  outdir: "./wwwroot/dist/",
  outExtension: { ".js": ".dist.js" },
};

if (process.argv.includes("--watch")) {
  const ctx = await esbuild.context({
    ...sharedConfig,
    minify: false,
    sourcemap: true,
  });
  await ctx.watch();
  console.log("watching...");
} else {
  await esbuild.build({
    ...sharedConfig,
    minify: true,
    sourcemap: false,
  });
}

import { glob } from "glob";
import terser from "@rollup/plugin-terser";
import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import replace from "@rollup/plugin-replace";
import del from "rollup-plugin-delete";

const production = !process.env.ROLLUP_WATCH;

export default {
  input: Object.fromEntries(
    glob
      .sync("**/*.razor.ts")
      .map((file) => [file.slice(0, file.length - ".razor.ts".length), file]),
  ),
  output: {
    sourcemap: !production,
    format: "es",
    dir: "./wwwroot/dist/",
  },
  plugins: [
    del({ targets: "./wwwroot/dist/*", runOnce: true }),
    replace({
      preventAssignment: true,
      "process.env.NODE_ENV": JSON.stringify(
        production ? "production" : "development",
      ),
    }),
    resolve({ browser: true }),
    commonjs(),
    typescript({
      sourceMap: !production,
      inlineSources: !production,
    }),
    production && terser(),
  ],
};

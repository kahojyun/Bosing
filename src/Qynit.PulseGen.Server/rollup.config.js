import { glob } from "glob";
import terser from "@rollup/plugin-terser";
import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import path from "path";

// eslint-disable-next-line no-undef
const production = !process.env.ROLLUP_WATCH;

export default {
  input: Object.fromEntries(
    glob
      .sync("**/*.razor.ts")
      .map((file) => [
        file.slice(0, file.length - path.extname(file).length),
        file,
      ]),
  ),
  output: {
    sourcemap: production ? false : "inline",
    format: "es",
    dir: "./",
  },
  plugins: [
    resolve({ browser: true }),
    commonjs(),
    typescript({
      sourceMap: !production,
      inlineSources: !production,
    }),
    production && terser(),
  ],
};

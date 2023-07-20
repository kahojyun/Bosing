import { globbySync } from 'globby';
import terser from "@rollup/plugin-terser";
import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import css from 'rollup-plugin-css-only';
import path from 'path';

const production = !process.env.ROLLUP_WATCH;

export default {
  input: globbySync(['Shared/**/*.ts', 'Pages/**/*.ts']),
  output: {
    sourcemap: !production,
    format: 'es',
    dir: './',
    entryFileNames: ({ facadeModuleId }) => {

      let root = path.resolve('.');
      let filePath = path.parse(facadeModuleId.substr(-(facadeModuleId.length - root.length) + 1));

      return `${filePath.dir}/[name].js`;
    },
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

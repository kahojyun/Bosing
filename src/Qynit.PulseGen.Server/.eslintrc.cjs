/* eslint-env node */
module.exports = {
  root: true,
  overrides: [
    {
      files: ["**/*.razor.ts"],
      extends: [
        "eslint:recommended",
        "plugin:@typescript-eslint/recommended-type-checked",
        "plugin:@typescript-eslint/stylistic-type-checked",
      ],
      parser: "@typescript-eslint/parser",
      plugins: ["@typescript-eslint"],
      parserOptions: {
        project: true,
        tsconfigRootDir: ".",
      },
    },
  ],
};

// 20 rules that exist identically in eslint, oxlint, and starlint.
// Used for fair head-to-head performance comparison.
import tsParser from "@typescript-eslint/parser";

export default [
  {
    files: ["**/*.{js,jsx,ts,tsx,mjs,cjs}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
    rules: {
      "for-direction": "error",
      "getter-return": "error",
      "no-cond-assign": "error",
      "no-console": "warn",
      "no-constant-condition": "error",
      "no-debugger": "error",
      "no-dupe-keys": "error",
      "no-duplicate-case": "error",
      "no-empty": "error",
      "no-empty-character-class": "error",
      "no-extra-boolean-cast": "error",
      "no-func-assign": "error",
      "no-sparse-arrays": "error",
      "no-unreachable": "error",
      "no-unsafe-finally": "error",
      "no-unsafe-negation": "error",
      "use-isnan": "error",
      "valid-typeof": "error",
      eqeqeq: "error",
      "no-var": "error",
    },
  },
];

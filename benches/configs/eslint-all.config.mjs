// ESLint config enabling ALL available rules from all plugins.
// Used for "all-rules" benchmark scenario — maximum rule load per tool.
//
// NOTE: typescript-eslint type-checked rules are excluded because neither
// starlint nor oxlint perform TypeScript type-checking. Including them would
// unfairly penalize eslint by running the full TS compiler on every file.
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactPlugin from "eslint-plugin-react";
import jsxA11yPlugin from "eslint-plugin-jsx-a11y";
import jestPlugin from "eslint-plugin-jest";
import promisePlugin from "eslint-plugin-promise";
import nodePlugin from "eslint-plugin-n";
import jsdocPlugin from "eslint-plugin-jsdoc";
import importPlugin from "eslint-plugin-import-x";
import reactHooksPlugin from "eslint-plugin-react-hooks";

// Enable every rule from a plugin as "warn", skipping rules that need type info.
function allRules(prefix, plugin) {
  return Object.fromEntries(
    Object.entries(plugin.rules)
      .filter(([, rule]) => !rule?.meta?.docs?.requiresTypeChecking)
      .map(([name]) => [`${prefix}/${name}`, "warn"]),
  );
}

export default [
  // ── Core ESLint: all 199 rules ────────────────────────────────────────
  js.configs.all,

  // ── TypeScript: strict (non-type-checked) ─────────────────────────────
  ...tseslint.configs.strict,

  // ── File targeting + parser ───────────────────────────────────────────
  {
    files: ["**/*.{js,jsx,ts,tsx,mjs,cjs}"],
    plugins: {
      react: reactPlugin,
      "react-hooks": reactHooksPlugin,
      "jsx-a11y": jsxA11yPlugin,
      jest: jestPlugin,
      promise: promisePlugin,
      n: nodePlugin,
      jsdoc: jsdocPlugin,
      "import-x": importPlugin,
    },
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        ecmaFeatures: { jsx: true },
        ecmaVersion: "latest",
        sourceType: "module",
      },
    },
    settings: {
      react: { version: "19" },
      jest: { version: 29 },
    },
    rules: {
      // Enable ALL rules from each plugin
      ...allRules("react", reactPlugin),
      ...allRules("react-hooks", reactHooksPlugin),
      ...allRules("jsx-a11y", jsxA11yPlugin),
      ...allRules("jest", jestPlugin),
      ...allRules("promise", promisePlugin),
      ...allRules("n", nodePlugin),
      ...allRules("jsdoc", jsdocPlugin),
      ...allRules("import-x", importPlugin),

      // Downgrade or disable rules that crash on certain files
      "no-unused-vars": "off",
      "no-undef": "warn",

      // eslint-plugin-n rules that crash without tsconfig or type info
      "n/no-sync": "off",
      "n/file-extension-in-import": "off",
      "n/no-extraneous-import": "off",
      "n/no-extraneous-require": "off",
      "n/no-hide-core-modules": "off",
      "n/no-missing-import": "off",
      "n/no-missing-require": "off",
      "n/no-restricted-import": "off",
      "n/no-restricted-require": "off",
      "n/no-unpublished-import": "off",
      "n/no-unpublished-require": "off",
    },
  },
];

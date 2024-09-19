import globals from "globals";
import js from "@eslint/js";
import prettier from "eslint-config-prettier";

export default [
  {
    rules: { "global-require": "off" },
    languageOptions: {
      globals: {
        ...globals.node,
        ...globals.jest,
      },
    },
  },
  prettier,
];

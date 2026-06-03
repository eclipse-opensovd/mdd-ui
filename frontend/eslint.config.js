// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0

import js from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginVue from "eslint-plugin-vue";
import vueTsConfig from "@vue/eslint-config-typescript";
import vuePrettierConfig from "@vue/eslint-config-prettier";

export default tseslint.config(
  {
    ignores: ["dist/", "node_modules/"],
  },
  js.configs.recommended,
  ...tseslint.configs.strict,
  ...tseslint.configs.stylistic,
  ...pluginVue.configs["flat/recommended"],
  ...vueTsConfig(),
  vuePrettierConfig,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "@typescript-eslint/consistent-type-imports": "error",
      "@typescript-eslint/no-non-null-assertion": "error",

      "vue/require-default-prop": "error",
      "vue/no-unused-vars": "error",
      "vue/component-name-in-template-casing": ["error", "PascalCase"],
      "vue/html-self-closing": [
        "error",
        { html: { void: "always", normal: "always", component: "always" } },
      ],

      "no-console": "warn",
      "no-debugger": "error",
      eqeqeq: ["error", "always"],
      "no-var": "error",
      "prefer-const": "error",
    },
  },
);

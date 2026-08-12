import js from '@eslint/js'
import vitest from '@vitest/eslint-plugin'
import vue from 'eslint-plugin-vue'
import tseslint from 'typescript-eslint'

// Browser globals used throughout the frontend (DOM types, timers). Listed
// explicitly rather than pulling in the `globals` package for a dozen names.
const browserGlobals = {
  window: 'readonly',
  document: 'readonly',
  console: 'readonly',
  setTimeout: 'readonly',
  clearTimeout: 'readonly',
  HTMLElement: 'readonly',
  HTMLInputElement: 'readonly',
  HTMLDialogElement: 'readonly',
  Event: 'readonly',
  MouseEvent: 'readonly',
  KeyboardEvent: 'readonly',
  Node: 'readonly',
}

export default tseslint.config(
  { ignores: ['dist/**', 'node_modules/**'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs['flat/recommended'],
  {
    languageOptions: {
      globals: browserGlobals,
    },
  },
  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    rules: {
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      // Stylistic-only rules that conflict with this codebase's deliberate
      // multi-attribute-per-line formatting; not a code quality signal.
      'vue/max-attributes-per-line': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/html-self-closing': 'off',
    },
  },
  {
    files: ['tests/**/*.spec.ts'],
    plugins: { vitest },
    rules: {
      ...vitest.configs.recommended.rules,
    },
  },
)

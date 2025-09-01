// ESLint Flat Config - Modern Best Practices with Next.js Integration
import js from '@eslint/js'
import tseslint from '@typescript-eslint/eslint-plugin'
import tsparser from '@typescript-eslint/parser'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import nextPlugin from 'eslint-config-next'

export default [
  // Base recommended configurations
  js.configs.recommended,
  
  // TypeScript configuration
  {
    name: 'typescript-setup',
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true,
        },
        project: ['./tsconfig.json'],
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
    },
    rules: {
      // TypeScript-specific rules
      '@typescript-eslint/no-unused-vars': ['error', { 
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_',
      }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-non-null-assertion': 'warn',
      '@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
      '@typescript-eslint/no-use-before-define': ['error', { 
        functions: false, 
        classes: true, 
        variables: true,
        typedefs: false, 
      }],
      '@typescript-eslint/strict-boolean-expressions': ['error', {
        allowString: false,
        allowNumber: false,
        allowNullableObject: false,
      }],
    },
  },

  // React configuration
  {
    name: 'react-setup',
    files: ['**/*.{jsx,tsx}'],
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      // React Hooks rules
      ...reactHooks.configs.recommended.rules,
      
      // React Refresh rules for development
      'react-refresh/only-export-components': ['warn', { 
        allowConstantExport: true, 
      }],
    },
  },

  // Next.js configuration
  {
    name: 'nextjs-setup',
    files: ['**/*.{js,jsx,ts,tsx}'],
    plugins: {
      'next': nextPlugin,
    },
    rules: {
      // Next.js recommended rules
      ...nextPlugin.configs.recommended.rules,
    },
  },

  // Additional Next.js specific overrides
  {
    name: 'nextjs-overrides',
    files: ['**/*.{js,jsx,ts,tsx}'],
    rules: {
      // Additional rules not covered by eslint-config-next
      'prefer-const': 'error',
      'no-var': 'error',
      'no-console': ['warn', { allow: ['warn', 'error'] }],
      'eqeqeq': ['error', 'always'],
      'curly': ['error', 'all'],
    },
  },

  // General JavaScript/TypeScript rules
  {
    name: 'general-rules',
    files: ['**/*.{js,jsx,ts,tsx}'],
    rules: {
      // Code quality
      'no-unused-vars': 'off', // Handled by TypeScript
      'no-undef': 'off', // Handled by TypeScript
      'prefer-const': 'error',
      'no-var': 'error',
      'object-shorthand': 'error',
      'prefer-template': 'error',
      'template-curly-spacing': ['error', 'never'],
      
      // Style consistency
      'quotes': ['error', 'single', { avoidEscape: true }],
      'semi': ['error', 'never'],
      'comma-dangle': ['error', 'always-multiline'],
      'indent': ['error', 2, { SwitchCase: 1 }],
      'linebreak-style': ['error', 'unix'],
      
      // Best practices
      'no-duplicate-imports': 'error',
      'no-useless-rename': 'error',
      'no-useless-computed-key': 'error',
      'no-useless-constructor': 'error',
      'prefer-destructuring': ['error', {
        array: true,
        object: true,
      }],
      
      // Error prevention
      'no-unreachable': 'error',
      'no-constant-condition': 'error',
      'no-dupe-keys': 'error',
      'no-duplicate-case': 'error',
      'no-empty': ['error', { allowEmptyCatch: true }],
      'no-extra-boolean-cast': 'error',
      'no-func-assign': 'error',
      'no-invalid-regexp': 'error',
      'no-irregular-whitespace': 'error',
      'no-obj-calls': 'error',
      'no-sparse-arrays': 'error',
      'no-unexpected-multiline': 'error',
      'use-isnan': 'error',
      'valid-typeof': 'error',
    },
  },

  // Test files configuration
  {
    name: 'test-files',
    files: ['**/*.test.{js,jsx,ts,tsx}', '**/*.spec.{js,jsx,ts,tsx}'],
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  },

  // Configuration files
  {
    name: 'config-files',
    files: ['*.config.{js,ts}', '*.config.*.{js,ts}'],
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-var-requires': 'off',
    },
  },

  // Ignore patterns
  {
    name: 'ignores',
    ignores: [
      'node_modules/**',
      '.next/**',
      'out/**',
      'dist/**',
      'build/**',
      '.vercel/**',
      '*.min.js',
      'coverage/**',
    ],
  },

  // Linter options
  {
    name: 'linter-options',
    linterOptions: {
      noInlineConfig: false,
      reportUnusedDisableDirectives: 'warn',
    },
  },
]
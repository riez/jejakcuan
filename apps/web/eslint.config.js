import svelte from 'eslint-plugin-svelte';
import tsParser from '@typescript-eslint/parser';

export default [
  {
    ignores: [
      '.svelte-kit/**',
      'build/**',
      'dist/**',
      'node_modules/**',
    ],
  },
  ...svelte.configs['flat/recommended'],
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: tsParser,
      },
    },
  },
  {
    files: ['**/*.ts'],
    languageOptions: {
      parser: tsParser,
    },
  },
];

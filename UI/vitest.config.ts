import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals:     true,
    environment: 'jsdom',
    setupFiles:  ['src/test-setup.ts'],
    include:     ['src/**/*.spec.ts'],
    exclude:     ['node_modules', 'dist', '.angular'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'html'],
      include:  ['src/app/**/*.ts'],
      exclude:  ['src/app/**/*.spec.ts', 'src/main.ts', 'src/environments/**'],
      thresholds: {
        lines:     80,
        functions: 80,
        branches:  70,
      },
    },
  },
});

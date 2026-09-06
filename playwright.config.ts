import { defineConfig } from 'playwright/test';

export default defineConfig({
  testDir: './tests/browser',
  timeout: 60_000,
  expect: { timeout: 8_000 },
  workers: 1,
  reporter: [['line'], ['json', { outputFile: 'test-results/claims.json' }]],
  use: {
    baseURL: 'http://127.0.0.1:4173',
    browserName: 'chromium',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'sh scripts/run-claim-server.sh',
    url: 'http://127.0.0.1:4173/health',
    timeout: 120_000,
    reuseExistingServer: false,
  },
});

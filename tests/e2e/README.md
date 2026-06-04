# E2E Tests

End-to-end tests using [WebdriverIO](https://webdriver.io/) with
[tauri-driver](https://crates.io/crates/tauri-driver) to drive the real desktop app.

## Prerequisites

1. **Build the Tauri app** in release mode:
   ```bash
   npm run tauri build
   ```

2. **Install tauri-driver** (Rust binary that acts as a WebDriver server):
   ```bash
   cargo install tauri-driver
   ```

3. **Install E2E dependencies** (separate from the main devDependencies to keep CI fast):
   ```bash
   npm install --save-dev \
     @wdio/cli@^9 \
     @wdio/local-runner@^9 \
     @wdio/mocha-framework@^9 \
     @wdio/spec-reporter@^9 \
     webdriverio@^9
   ```

4. **Linux only**: E2E requires a display. Run tests with `xvfb-run`:
   ```bash
   xvfb-run --auto-servernum npm run test:e2e
   ```

## Running

```bash
# macOS / Windows
npm run test:e2e

# Linux
xvfb-run --auto-servernum npm run test:e2e
```

## CI

E2E tests run automatically on release tags and can be triggered manually via the
**E2E Tests** workflow in GitHub Actions. They are intentionally excluded from the
per-PR CI to avoid the cost of a full Tauri build on every commit.

## Test structure

```
tests/e2e/
├── wdio.conf.ts       # WebdriverIO configuration
├── specs/
│   └── smoke.spec.ts  # Basic navigation smoke tests
└── README.md          # This file
```

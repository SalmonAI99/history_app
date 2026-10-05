// End-to-end test runner for the bookreader app.
//
// Usage:
//   1. build the app:        cargo leptos build     (from the project root)
//   2. one-time setup:       cd e2e && npm install && npx playwright install chromium
//   3. run the tests:        npm test               (from this e2e/ folder)
//
// What it does:
//   - starts the server on port 3100 with a THROWAWAY database
//     (your normal bookreader.db is never touched)
//   - runs each *.test.js suite in a real (headless) browser
//   - reports PASS/FAIL per suite and exits non-zero on failure

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { chromium } = require('playwright');

const ROOT = path.join(__dirname, '..');
const PORT = 3100;
const BIN = path.join(ROOT, 'target', 'debug', 'bookreader');
const ALL = ['./reader.test.js', './auth.test.js', './posts.test.js', './files.test.js'];
// `node run-all.js reader` -> run only suites whose name contains "reader"
const filter = process.argv[2];
const SUITES = filter ? ALL.filter((s) => s.includes(filter)) : ALL;

async function waitForServer(url, tries = 60) {
  for (let i = 0; i < tries; i++) {
    try {
      const r = await fetch(url);
      if (r.ok) return;
    } catch { /* not up yet */ }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error('server did not start on ' + url);
}

(async () => {
  if (!fs.existsSync(BIN)) {
    console.error('server binary not found — run `cargo leptos build` first');
    process.exit(1);
  }

  // Leptos 0.8's hydration snippet calls init('/pkg/NAME_bg.wasm'), but
  // cargo-leptos 0.3.9 writes the file as NAME.wasm -> 404 -> hydration
  // silently dies. Make the _bg alias exist (identical bytes).
  const pkg = path.join(__dirname, '..', 'target', 'site', 'pkg');
  const wasm = path.join(pkg, 'bookreader.wasm');
  const wasmBg = path.join(pkg, 'bookreader_bg.wasm');
  if (fs.existsSync(wasm) && !fs.existsSync(wasmBg)) {
    fs.copyFileSync(wasm, wasmBg);
  }

  const dbFile = path.join(os.tmpdir(), `bookreader_e2e_${process.pid}.db`);
  // Stage 10: uploads also get a throwaway folder, cleaned up after.
  const uploadDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bookreader_e2e_storage_'));
  const server = spawn(BIN, [], {
    cwd: ROOT,
    stdio: 'ignore',
    env: {
      ...process.env,
      DATABASE_URL: `sqlite:${dbFile}?mode=rwc`,
      LEPTOS_SITE_ADDR: `127.0.0.1:${PORT}`,
      // LEPTOS_* env vars put Leptos in "env-only" config mode — it stops
      // reading Cargo.toml and output-name falls back to "" (serving
      // /pkg/.js instead of /pkg/bookreader.js). Set it explicitly.
      LEPTOS_OUTPUT_NAME: 'bookreader',
      UPLOAD_DIR: uploadDir,
    },
  });

  let failures = 0;
  try {
    await waitForServer(`http://127.0.0.1:${PORT}/`);

    for (const suite of SUITES) {
      const run = require(path.join(__dirname, suite));
      const browser = await chromium.launch();
      const context = await browser.newContext({ baseURL: `http://127.0.0.1:${PORT}` });
      try {
        await run(context);
        console.log(`PASS  ${suite}`);
      } catch (e) {
        failures++;
        console.error(`FAIL  ${suite}\n      ${e.message}`);
      }
      await browser.close();
    }
  } finally {
    server.kill();
    try { fs.unlinkSync(dbFile); } catch { /* ignore */ }
    try { fs.rmSync(uploadDir, { recursive: true, force: true }); } catch { /* ignore */ }
  }

  console.log(failures ? `\n${failures} suite(s) failed` : '\nall suites passed');
  process.exit(failures ? 1 : 0);
})();

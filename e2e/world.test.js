// Suite: Book World (2D grid multiplayer POC).
// Two browser contexts ("two players"): both sign up fresh here so the
// suite also runs standalone (`node run-all.js world`), and in the full
// chain after auth.test.js.
// Movement is optimistic+server, presence refreshes every 1.5s, so
// cross-player checks wait a few seconds.

function assert(cond, msg) {
  if (!cond) throw new Error('assert failed: ' + msg);
}

const settle = (page, ms = 4000) => page.waitForTimeout(ms);

async function signup(page, name) {
  await page.goto('/login', { waitUntil: 'domcontentloaded' });
  // Hydration signal: the WASM download finished (+ a moment to boot).
  // (Timings alone kept being too short as the bundle grew.)
  await page
    .waitForResponse((r) => r.url().includes('.wasm'), { timeout: 3000 })
    .catch(() => {}); // cached on later loads: fine, just don't hang
  await page.waitForTimeout(1500);
  await page.click('text=Create an account');
  await page.waitForTimeout(1000);
  await page.fill('input[name="username"]', name);
  await page.fill('input[name="password"]', 'secret123');
  await page.click('.auth-form button[type="submit"]');
  await page.waitForTimeout(3500); // let the signup POST finish
}

async function enterWorld(page) {
  await page.goto('/book_world', { waitUntil: 'domcontentloaded' });
  await settle(page);
}

module.exports = async function run(context) {
  const BASE = 'http://127.0.0.1:3100';
  const pageA = await context.newPage();
  const errors = [];
  pageA.on('pageerror', (err) => { errors.push(err.message); console.log('PAGEERR A:', err.message.slice(0, 150)); });
  pageA.on('console', (m) => { if (m.type() === 'error') console.log('CONSOLE A:', m.text().slice(0, 150)); });

  // Guest: the world asks for a login.
  await pageA.goto('/book_world', { waitUntil: 'domcontentloaded' });
  await settle(pageA);
  let main = await pageA.textContent('main');
  assert(main.includes('log in'), 'guest should see the login hint, got: ' + main.slice(0, 120));

  // Player A signs up and enters.
  await signup(pageA, 'world_parent');
  // TEMP DIAG: did A actually get logged in? and did the toggle flip?
  const errEl = await pageA.$('.error');
  console.log('DIAG A error element:', errEl ? await errEl.textContent() : '（none)');
  await pageA.goto('/', { waitUntil: 'domcontentloaded' });
  await pageA.waitForTimeout(2500);
  console.log('DIAG A user-nav:', JSON.stringify(await pageA.textContent('.user-nav')));
  await enterWorld(pageA);
  main = await pageA.textContent('main');
  assert(main.includes('(you)'), 'A should see themselves marked, got: ' + main.slice(0, 250));
  assert(await pageA.$('.game'), 'the grid field should render');
  assert(await pageA.$('.avatar.you'), 'A should have an avatar');

  // A greets with the one-click button -> bubble next to A.
  await pageA.click('.chat-bar.inline button[type="submit"]');
  await settle(pageA);
  main = await pageA.textContent('main');
  assert(main.includes('👋 Hello!'), "A's greeting should show, got: " + main.slice(0, 250));

  // A says a custom line too.
  await pageA.fill('.chat-bar input[type="text"]', 'hello grid');
  await pageA.click('.chat-bar:not(.inline) button[type="submit"]');
  await settle(pageA);
  main = await pageA.textContent('main');
  assert(main.includes('hello grid'), 'custom message should show');

  // A walks onto the bookshelf pad (spawn 3,3 -> shelf at 1,0:
  // Left x2, Up x3). The jump overlay must appear immediately.
  await pageA.click('.game');
  await pageA.keyboard.press('ArrowLeft');
  await pageA.keyboard.press('ArrowLeft');
  await pageA.keyboard.press('ArrowUp');
  await pageA.keyboard.press('ArrowUp');
  await pageA.keyboard.press('ArrowUp');
  await settle(pageA, 1500);
  const overlay = await pageA.$('.jump-overlay');
  assert(overlay, 'standing on the shelf should reveal the jump overlay');
  const href = await pageA.getAttribute('.jump-overlay a.button', 'href');
  assert(href && href.match(/^\/book\/\d+\/chapter\/1$/), 'jump link should point to the reader, got ' + href);

  // Player B signs up in a second context and enters.
  const ctxB = await context.browser().newContext({ baseURL: BASE });
  const pageB = await ctxB.newPage();
  pageB.on('pageerror', (err) => errors.push(err.message));
  await signup(pageB, 'world_friend');
  // TEMP DIAG: did B actually get logged in by the signup?
  await pageB.goto('/', { waitUntil: 'domcontentloaded' });
  await pageB.waitForTimeout(3000);
  console.log('DIAG B user-nav:', JSON.stringify(await pageB.textContent('.user-nav')));
  await enterWorld(pageB);

  // B should see player A (poll catches up) standing at the shelf.
  for (let i = 0; i < 5; i++) {
    const m = await pageB.textContent('main');
    if (m.includes('world_parent')) break;
    await pageB.waitForTimeout(2500);
  }
  let mainB = await pageB.textContent('main');
  assert(mainB.includes('world_parent'), 'B should see player A, page says: ' + mainB.slice(0, 300));

  // B greets back; after a poll cycle A should see B + the greeting.
  await pageB.click('.chat-bar.inline button[type="submit"]');
  for (let i = 0; i < 5; i++) {
    const m = await pageA.textContent('main');
    if (m.includes('world_friend') && m.includes('👋')) break;
    await pageA.waitForTimeout(2500);
  }
  main = await pageA.textContent('main');
  assert(main.includes('world_friend'), 'A should see player B');
  assert(main.includes('👋'), "A should see B's greeting bubble");

  assert(errors.length === 0, 'no browser errors, got: ' + errors.join(' | '));
  await pageA.close();
  await ctxB.close();
};

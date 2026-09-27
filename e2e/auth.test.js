// Suite: authentication (Stage 6).
// signup -> cookie persistence -> logout -> wrong password -> login.
// NOTE: the runner gives us a FRESH temporary database, so signing
//       up here never pollutes the development database.

function assert(cond, msg) {
  if (!cond) throw new Error('assert failed: ' + msg);
}

const settle = (page, ms = 2500) => page.waitForTimeout(ms);
const nav = (page) => page.evaluate(() => document.querySelector('.user-nav')?.textContent || '');

module.exports = async function run(context) {
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (err) => errors.push(err.message));

  // 1. Guests see "Log in"
  await page.goto('/', { waitUntil: 'networkidle' });
  await settle(page);
  assert((await nav(page)).includes('Log in'), 'guest nav should show "Log in"');

  // 2. Sign up a new parent account
  await page.click('.user-nav a');
  await settle(page, 1200);
  await page.click('text=Create an account');
  await page.fill('input[name="username"]', 'test_parent');
  await page.fill('input[name="password"]', 'secret123');
  await page.click('button[type="submit"]');
  await settle(page);
  assert((await nav(page)).includes('test_parent'), 'nav should show username after signup');

  // 3. Reloading keeps us logged in (session cookie)
  await page.reload({ waitUntil: 'networkidle' });
  await settle(page);
  assert((await nav(page)).includes('test_parent'), 'should stay logged in after reload');

  // 4. The session cookie is invisible to JavaScript (HttpOnly)
  const cookie = await page.evaluate(() => document.cookie);
  assert(cookie === '', 'session cookie must be HttpOnly (document.cookie was: ' + cookie + ')');

  // 5. Logout returns the nav to "Log in"
  await page.click('text=Log out');
  await settle(page);
  assert((await nav(page)).includes('Log in'), 'nav should show "Log in" after logout');

  // 6. Wrong password shows an error, does NOT log in
  await page.click('.user-nav a');
  await settle(page, 1200);
  await page.fill('input[name="username"]', 'test_parent');
  await page.fill('input[name="password"]', 'wrongpass');
  await page.click('button[type="submit"]');
  await settle(page);
  const err = await page.textContent('.error').catch(() => null);
  assert(err && err.includes('Wrong username or password'), 'wrong password should show the error, got: ' + err);

  // 7. Correct password logs in
  await page.fill('input[name="username"]', 'test_parent');
  await page.fill('input[name="password"]', 'secret123');
  await page.click('button[type="submit"]');
  await settle(page);
  assert((await nav(page)).includes('test_parent'), 'should log in with correct password');

  assert(errors.length === 0, 'no browser errors, got: ' + errors.join(' | '));
  await page.close();
};

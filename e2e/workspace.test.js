// Suite: the workspace (Stage 9).
// Progress auto-save + library "Continue", chapter notes, writings,
// ownership rules. Runs after auth.test.js (test_parent exists) and
// posts.test.js — test_parent is the admin, logs in fresh each suite.

function assert(cond, msg) {
  if (!cond) throw new Error('assert failed: ' + msg);
}

const settle = (page, ms = 2500) => page.waitForTimeout(ms);

module.exports = async function run(context) {
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (err) => errors.push(err.message));

  // 1. Guest reader: NO note box, but a login hint
  await page.goto('/book/1/chapter/1', { waitUntil: 'networkidle' });
  await settle(page);
  const guestMain = await page.textContent('main');
  assert(!guestMain.includes('Your notes for this chapter'), 'guest must NOT see the note box');
  assert(guestMain.includes('save notes and track your progress'), 'guest should see the login hint');

  // 2. Guest workspace: asked to log in
  await page.goto('/workspace', { waitUntil: 'networkidle' });
  await settle(page);
  assert((await page.textContent('main')).includes('log in'), 'guest should be asked to log in at /workspace');

  // 3. Log in as test_parent
  await page.goto('/login', { waitUntil: 'networkidle' });
  await settle(page, 1200);
  await page.fill('input[name="username"]', 'test_parent');
  await page.fill('input[name="password"]', 'secret123');
  await page.click('.auth-form button[type="submit"]');
  await settle(page, 3000);

  // 4. Read chapter 2 (progress saves automatically)
  await page.goto('/book/1/chapter/2', { waitUntil: 'networkidle' });
  await settle(page);

  // 5. Library now offers "Continue · Chapter 2"
  await page.goto('/library', { waitUntil: 'networkidle' });
  await settle(page);
  const lib = await page.textContent('main');
  assert(lib.includes('Chapter 2'), 'library should offer Continue at Chapter 2, got: ' + lib.slice(0, 140));

  // 6. "Continue" takes us right back to chapter 2
  await page.click('text=Continue');
  await settle(page);
  assert(page.url().includes('/book/1/chapter/2'), 'Continue should land on chapter 2');

  // 7. Save a note in the reader -> textarea clears
  assert((await page.textContent('main')).includes('Your notes for this chapter'), 'note box should show when logged in');
  await page.fill('.note-form textarea', 'Emma is brave. I like this library!');
  await page.click('.note-form button[type="submit"]');
  await settle(page);
  assert((await page.inputValue('.note-form textarea')) === '', 'textarea should clear after saving');

  // 8. Workspace lists the note with its chapter + content
  await page.goto('/workspace', { waitUntil: 'networkidle' });
  await settle(page);
  const ws = await page.textContent('main');
  assert(ws.includes('Chapter 2'), 'workspace should show the note with its chapter, got: ' + ws.slice(0, 200));
  assert(ws.includes('Emma is brave'), 'workspace should show the note content');

  // 9. Create a writing
  await page.fill('input[name="title"]', 'My first story idea');
  await page.fill('textarea[name="content"]', 'A dragon who is afraid of fire.');
  await page.click('.post-form button[type="submit"]');
  await settle(page);
  assert((await page.textContent('main')).includes('My first story idea'), 'writing should appear in the list');

  // 10. Delete the note -> it disappears (and the writing survives)
  const row = page.locator('li', { hasText: 'Emma is brave' });
  await row.locator('button[type="submit"]').click();
  await settle(page);
  const after = await page.textContent('main');
  assert(!after.includes('Emma is brave'), 'deleted note should be gone');
  assert(after.includes('My first story idea'), 'writing should survive deleting the note');

  assert(errors.length === 0, 'no browser errors, got: ' + errors.join(' | '));
  await page.close();
};

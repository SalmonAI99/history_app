// Suite: the reading experience (Stages 3-5).
// Library -> reader -> chapter navigation -> URL behavior -> error states.

function assert(cond, msg) {
  if (!cond) throw new Error('assert failed: ' + msg);
}

// Hydration needs a moment after each navigation.
const settle = (page, ms = 2000) => page.waitForTimeout(ms);

module.exports = async function run(context) {
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (err) => errors.push(err.message));

  // 1. Library lists the seeded book (library lives at /library since Stage 8)
  await page.goto('/library', { waitUntil: 'domcontentloaded' });
  await settle(page);
  const bookLink = await page.textContent('.book-list a').catch(() => null);
  assert(bookLink && bookLink.length > 0, 'library should list a book');

  // 2. Clicking it opens chapter 1 at a real URL
  await page.click('.book-list a');
  await settle(page);
  assert(page.url().includes('/book/1/chapter/1'), 'URL should be /book/1/chapter/1, got ' + page.url());
  assert((await page.textContent('#chapter-title')) === 'Chapter 1', 'should show Chapter 1');
  assert((await page.textContent('#page-number')) === '1 / 2', 'counter should be 1 / 2');

  // 3. "Next" goes to chapter 2 - by CHANGING THE URL
  await page.click('text=Next →');
  await settle(page);
  assert(page.url().includes('/book/1/chapter/2'), 'URL should move to chapter 2');
  assert((await page.textContent('#chapter-title')) === 'Chapter 2', 'should show Chapter 2');

  // 4. The browser back button works (URL-is-state!)
  await page.goBack();
  await settle(page);
  assert((await page.textContent('#chapter-title')) === 'Chapter 1', 'back button should return to Chapter 1');

  // 5. A deep link loads its chapter directly (SSR)
  await page.goto('/book/1/chapter/2', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('#chapter-title')) === 'Chapter 2', 'deep link should show Chapter 2');

  // 6. A chapter that doesn't exist shows the error message
  await page.goto('/book/1/chapter/99', { waitUntil: 'domcontentloaded' });
  await settle(page);
  const body = await page.textContent('.reader');
  assert(body.includes('could not be found'), 'bad chapter should show the error text');

  assert(errors.length === 0, 'no browser errors, got: ' + errors.join(' | '));
  await page.close();
};

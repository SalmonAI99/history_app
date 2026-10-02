// Suite: the public site + admin publishing (Stage 8).
// Marketing pages, blog/news lists, Markdown rendering, admin gates.
// Runs AFTER auth.test.js, so 'test_parent' (the admin) already exists
// in this test-run's database.

function assert(cond, msg) {
  if (!cond) throw new Error('assert failed: ' + msg);
}

const settle = (page, ms = 2000) => page.waitForTimeout(ms);

module.exports = async function run(context) {
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (err) => errors.push(err.message));

  // 1. Home: hero + a "Latest" section with the seeded posts
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await settle(page);
  const hero = await page.textContent('.hero h2').catch(() => null);
  assert(hero && hero.length > 0, 'home should have a hero headline');
  const latest = await page.textContent('main');
  assert(latest.includes('欢迎') || latest.includes('秋季'), 'home Latest should show a seeded (Chinese) post');

  // 2. /blog lists the seeded welcome post
  await page.goto('/blog', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('main')).includes('欢迎'), '/blog should list the welcome post');

  // 3. Post detail renders Markdown as real HTML (an <h2> inside content)
  await page.goto('/blog/welcome', { waitUntil: 'domcontentloaded' });
  await settle(page);
  const heading = await page.textContent('.post-content h2').catch(() => null);
  assert(heading && heading.includes('欢迎'), 'post Markdown should render an <h2>, got: ' + heading);

  // 4. /news lists the seeded news post
  await page.goto('/news', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('main')).includes('秋季'), '/news should list the course news');

  // 5. Teacher page loads
  await page.goto('/teacher', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('main')).includes('Teacher'), 'teacher page should load');

  // 6. Guest hitting /admin is asked to log in
  await page.goto('/admin', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('main')).includes('log in'), 'guest should be asked to log in at /admin');

  // 7. Log in as the admin (test_parent exists from the auth suite)
  await page.goto('/login', { waitUntil: 'domcontentloaded' });
  await settle(page, 1200);
  await page.fill('input[name="username"]', 'test_parent');
  await page.fill('input[name="password"]', 'secret123');
  await page.click('button[type="submit"]');
  await settle(page, 3000);
  assert((await page.textContent('.user-nav')).includes('Admin'), 'admin should see the Admin link in nav');

  // 8. Publish a new post via the admin form
  //    (careful: click the PUBLISH button inside .post-form — a plain
  //    'button[type="submit"]' would hit "Log out" in the header first!)
  await page.goto('/admin', { waitUntil: 'domcontentloaded' });
  await settle(page);
  await page.fill('input[name="slug"]', 'e2e-test-post');
  await page.fill('input[name="title"]', 'E2E Test Post');
  await page.fill('textarea[name="content"]', '## Hello\n\nsome **bold** text');
  await page.click('.post-form button[type="submit"]');
  await settle(page);
  assert((await page.textContent('main')).includes('Published!'), 'should confirm the publish');

  // 9. The new post is live at /blog/e2e-test-post
  await page.goto('/blog/e2e-test-post', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('.post-content h2')).includes('Hello'), 'new post should render its Markdown heading');
  assert((await page.innerHTML('.post-content')).includes('<strong>bold</strong>'), '**bold** should become <strong>');

  // 10. Delete it again via the admin list -> detail page then errors
  await page.goto('/admin', { waitUntil: 'domcontentloaded' });
  await settle(page);
  const row = page.locator('li', { hasText: 'E2E Test Post' });
  await row.locator('button[type="submit"]').click();
  await settle(page);
  await page.goto('/blog/e2e-test-post', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('main')).includes("doesn't exist"), 'deleted post should 404 politely');

  // 11. A non-admin (student) sees the admin gate, NOT the form
  await page.goto('/login', { waitUntil: 'domcontentloaded' });
  await settle(page, 1200);
  await page.click('text=Create an account');
  await page.fill('input[name="username"]', 'student_2');
  await page.fill('input[name="password"]', 'secret123');
  // scope to the auth form — while logged in, a bare 'button[type=submit]'
  // clicks "Log out" in the header (it sits before main in the DOM)
  await page.click('.auth-form button[type="submit"]');
  await settle(page, 3000);
  await page.goto('/admin', { waitUntil: 'domcontentloaded' });
  await settle(page);
  assert((await page.textContent('main')).includes('admin) only'), 'student should see the admin gate');

  assert(errors.length === 0, 'no browser errors, got: ' + errors.join(' | '));
  await page.close();
};

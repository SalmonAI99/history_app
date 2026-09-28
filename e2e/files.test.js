// Suite: files & video (Stage 10).
// Uploads, the gated /files route, range requests, deletion.
// Runs after workspace.test.js — test_parent (admin) exists.

const BASE = 'http://127.0.0.1:3100';

function assert(cond, msg) {
  if (!cond) throw new Error('assert failed: ' + msg);
}

const settle = (page, ms = 2500) => page.waitForTimeout(ms);

async function login(page, name) {
  await page.goto('/login', { waitUntil: 'networkidle' });
  await settle(page, 1200);
  await page.fill('input[name="username"]', name);
  await page.fill('input[name="password"]', 'secret123');
  await page.click('.auth-form button[type="submit"]');
  await settle(page, 3000);
}

async function logout(page) {
  await page.goto('/', { waitUntil: 'networkidle' });
  await settle(page, 1200);
  await page.click('.user-nav button[type="submit"]');
  await settle(page);
}

module.exports = async function run(context) {
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (err) => errors.push(err.message));

  // 1. Guest hits /files?id=1 directly -> 401, no bytes.
  const guest = await page.request.get(`${BASE}/files?id=1`);
  assert(guest.status() === 401, 'guest should get 401, got ' + guest.status());

  // 2. Guests see a login hint on both file pages.
  await page.goto('/materials', { waitUntil: 'networkidle' });
  await settle(page);
  assert((await page.textContent('main')).includes('log in'), 'materials should gate guests');
  await page.goto('/member', { waitUntil: 'networkidle' });
  await settle(page);
  assert((await page.textContent('main')).includes('log in'), 'videos should gate guests');

  // 3. Admin logs in, uploads a text worksheet from /materials.
  await login(page, 'test_parent');
  await page.goto('/materials', { waitUntil: 'networkidle' });
  await settle(page);
  const noteText = 'Reading homework: chapters 1 and 2.\nWrite two sentences about Emma.';
  await page.setInputFiles('.upload-form input[type="file"]', {
    name: 'homework.txt', mimeType: 'text/plain', buffer: Buffer.from(noteText),
  });
  await Promise.all([
    page.waitForNavigation({ waitUntil: 'networkidle' }),
    page.click('.upload-form button[type="submit"]'),
  ]);
  await settle(page);
  const mats = await page.textContent('main');
  assert(mats.includes('homework.txt'), 'material should appear after upload, got: ' + mats.slice(0, 200));

  // 4. The download link serves the exact bytes back.
  const href = await page.getAttribute('.doc-item .doc-heading a', 'href');
  assert(href && href.startsWith('/files?id='), 'download link should be /files?id=..., got ' + href);
  const dl = await page.request.get(BASE + href);
  assert(dl.status() === 200, 'download should be 200, got ' + dl.status());
  assert((await dl.text()) === noteText, 'downloaded bytes must match what we uploaded');

  // 5. Range request -> 206 Partial Content with the right slice.
  const ranged = await page.request.get(BASE + href, { headers: { Range: 'bytes=0-9' } });
  assert(ranged.status() === 206, 'range request should be 206, got ' + ranged.status());
  const body = await ranged.body();
  assert(body.length === 10 && body.toString() === noteText.slice(0, 10),
    'range body should be the first 10 bytes, got ' + body.toString());

  // 6. Upload a video on /member -> player appears with a gated src.
  await page.goto('/member', { waitUntil: 'networkidle' });
  await settle(page);
  await page.setInputFiles('.upload-form input[type="file"]', {
    name: 'lesson1.mp4', mimeType: 'video/mp4', buffer: Buffer.from('FAKE-MP4-BYTES-FOR-TESTING'),
  });
  await Promise.all([
    page.waitForNavigation({ waitUntil: 'networkidle' }),
    page.click('.upload-form button[type="submit"]'),
  ]);
  await settle(page);
  const vids = await page.textContent('main');
  assert(vids.includes('lesson1.mp4'), 'video should appear after upload, got: ' + vids.slice(0, 200));
  const videoSrc = await page.getAttribute('video', 'src');
  assert(videoSrc && videoSrc.startsWith('/files?id='), 'player src should be gated, got ' + videoSrc);

  // 7. The video is NOT on the materials page, and vice versa.
  await page.goto('/materials', { waitUntil: 'networkidle' });
  await settle(page);
  assert(!(await page.textContent('main')).includes('lesson1.mp4'), 'videos must not appear on materials');

  // 8. Range request against the video -> 206.
  const vr = await page.request.get(BASE + videoSrc, { headers: { Range: 'bytes=0-4' } });
  assert(vr.status() === 206, 'video range should be 206, got ' + vr.status());

  // 9. Unknown id -> 404.
  const missing = await page.request.get(`${BASE}/files?id=99999`);
  assert(missing.status() === 404, 'unknown id should be 404, got ' + missing.status());

  // 10. A non-admin student: can download materials, sees NO forms.
  await logout(page);
  await page.goto('/login', { waitUntil: 'networkidle' });
  await settle(page, 1200);
  await page.click('text=Sign up');
  await settle(page, 800);
  await page.fill('input[name="username"]', 'file_student');
  await page.fill('input[name="password"]', 'secret123');
  await page.click('.auth-form button[type="submit"]');
  await settle(page, 3000);
  await page.goto('/materials', { waitUntil: 'networkidle' });
  await settle(page);
  const stuMats = await page.textContent('main');
  assert(stuMats.includes('homework.txt'), 'student should see the material');
  assert(!(await page.$('.upload-form')), 'student must NOT see an upload form');
  assert(!(await page.$('.doc-item button')), 'student must NOT see delete buttons');
  const stuDl = await page.request.get(BASE + href);
  assert(stuDl.status() === 200, 'student can download materials');

  // 11. Back as admin: delete both files -> rows AND bytes gone.
  await logout(page);
  await login(page, 'test_parent');
  await page.goto('/materials', { waitUntil: 'networkidle' });
  await settle(page);
  await page.click('.doc-item button[type="submit"]'); // delete homework.txt
  await settle(page);
  assert(!(await page.textContent('main')).includes('homework.txt'), 'deleted material should vanish');
  const goneDl = await page.request.get(BASE + href);
  assert(goneDl.status() === 404, 'deleted file should 404, got ' + goneDl.status());

  await page.goto('/member', { waitUntil: 'networkidle' });
  await settle(page);
  await page.click('.doc-item button[type="submit"]'); // delete lesson1.mp4
  await settle(page);
  assert(!(await page.textContent('main')).includes('lesson1.mp4'), 'deleted video should vanish');

  assert(errors.length === 0, 'no browser errors, got: ' + errors.join(' | '));
  await page.close();
};

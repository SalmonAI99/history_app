# Debugging field manual — bookreader

How to figure out what's wrong when something breaks, tuned to THIS stack
(Leptos SSR + hydration, server functions, SQLite). Written after the real
bugs we hit building stages 1–8, so every case study is one of ours.

---

## The golden rules

1. **Evidence over theories.** Every hour I lost came from reasoning instead of
   looking. When stuck: stop thinking, go *look at the bytes and the behavior*.
2. **Read the actual error message. Really.** Rust and panics usually tell you
   the exact line and what's wrong ("called use_resolved_path outside a <Router>"
   literally said the header's `<A>` needed to be inside `<Router>`).
3. **Change ONE thing at a time.** Change two and you don't know which fixed it
   (or which broke it).
4. **The bug is almost always in our code, not the framework.**
5. **Make it smaller.** Can't find it? Delete/skip half the moving parts until
   the bug disappears, then add back until it returns. The last piece is guilty.
6. **Say it out loud.** Explaining the bug to someone (or a rubber duck, or an
   AI) forces the fuzzy assumption into words — that's usually where it hides.

## The loop

```
reproduce reliably  →  isolate which layer  →  get evidence from that layer
→  form ONE theory  →  test it  →  fix  →  verify with the test suites
```

Isolation is the key skill. Our stack has four layers, and each has its own
way to look inside:

```
BROWSER (WASM hydration, signals, view!)
   ↕ HTTP (server function POSTs, cookies)
SERVER (Axum, server functions, SSR HTML)
   ↕ SQL
DATABASE (bookreader.db — SQLite tables)
```

---

## The toolkit

### 0. Is the server even running / is the port taken?

Most common cause of "nothing works" or "my changes don't appear".

```bash
ss -tlnp | grep -E "3000|3001"     # who owns the ports?
ps aux | grep -E "leptos|bookreader" | grep -v grep
```

- Rules: ONE `cargo leptos watch` at a time. Never run `cargo leptos build`
  separately while `watch` is running — they corrupt each other's output
  (this caused a full day of "frozen page" confusion for us).
- After code changes: let `watch` rebuild, then **hard-refresh** the browser
  (Ctrl+Shift+R). WASM/JS URLs aren't cache-busted in dev.

### 1. What is the SERVER sending? (curl = X-ray vision)

```bash
curl -s http://127.0.0.1:3000/ | head -c 800        # raw SSR HTML
curl -s http://127.0.0.1:3000/ | grep -o 'Chapter 1' # is my content in it?
```

**If the raw HTML is wrong → the bug is server-side (render/query).**
**If the raw HTML is right but the page looks wrong → hydration/client-side.**

Two famous catches from our bugs:
- HTML showed `<button disabled="1">= total &gt;Next →</button>` — the `>=` in a
  `view!` attribute was parsed as HTML. (Rule: any attribute expression with
  `<` or `>` needs braces: `disabled=move || { a >= b }`.)
- HTML showed `Loading...` + a `<template>` — that's normal streaming SSR;
  the content arrives in a later chunk. Search the whole page before judging.

### 2. Calling a server function directly (demystify the "magic")

Server functions are just HTTP POSTs. Their URLs contain a hash suffix —
find it, then call it like the browser does:

```bash
# 1. find the real endpoint name
strings target/site/pkg/bookreader.wasm | grep -o 'get_chapter[0-9]*'

# 2. call it (args are form-encoded; add Accept for JSON back)
curl -s -X POST "http://127.0.0.1:3000/api/get_chapter3181626185042595084" \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -H "Accept: application/json" \
     -d 'index=1'
```

If curl works but the browser fails → client-side issue (serialization,
wrong arg types — e.g. `Option<String>` args can't go in a URL-encoded body;
use plain types). If curl also fails → server-side: check the server log next.

### 3. Server logs, telemetry, and panics

The server uses **structured logging** (`tracing`). Every HTTP request gets a
span (method, path, latency, status), and business events are logged inside
it. A real excerpt:

```
INFO  bookreader: listening on http://127.0.0.1:3000
DEBUG request{method=POST uri=/api/signup...}: started processing request
INFO  request{...}: new user registered   event="signup"     user=tel_test role=admin
DEBUG request{...}: finished processing request   latency=701 ms status=200
WARN  request{...}: wrong password        event="login_fail" user=tel_test
```

Business events to grep for: `signup`, `signup_fail`, `login`, `login_fail`,
`publish`, `delete_post` (payments join this list in Stage 11).

```bash
# control verbosity (default: info + HTTP request spans):
RUST_LOG=debug cargo leptos watch        # everything
RUST_LOG=info cargo leptos watch         # quieter, no per-request lines
# watch the business events only:
cargo leptos watch 2>&1 | grep 'event='
```

A panicking server function or crashed startup also prints to this log.
For full backtraces run the binary yourself (watch stopped):

```bash
RUST_BACKTRACE=1 ./target/debug/bookreader
```

Gotcha met once: `sqlite:bookreader.db` does NOT create a missing file —
needs `?mode=rwc` (already in our `db.rs`, remember it if you change paths).

### 4. What is the BROWSER doing? (F12 is your co-pilot)

- **Console** — hydration mismatches, WASM panics. Red text = start here.
- **Network tab** — click a button, watch the `POST /api/...` request:
  status 200? What did it return? (A 500 means the server function errored —
  the real message is usually in the server log.)
- **Application → Cookies** — is the `session` cookie there? HttpOnly?
  Expires 30 days out? Cookie missing after login = the Set-Cookie path.

### 5. What's in the DATABASE?

```bash
python3 -c "
import sqlite3
conn = sqlite3.connect('bookreader.db')
print(list(conn.execute('SELECT id, username, role FROM users')))
print(list(conn.execute('SELECT slug, kind, title FROM posts')))
"
```

Reset to factory state (re-seeds books + demo posts on next start):

```bash
rm bookreader.db
```

Caution: `books/book.txt` only imports when the DB is empty. Content edits
now belong in the DB (or the admin page for posts).

### 6. The test suites (your smoke detector)

```bash
cargo test --features ssr     # Rust: parser, DB layer, hashing — seconds
cd e2e && npm test            # browser: reads, auth, posts — needs
                              # `cargo leptos build` first
```

A failing test tells you WHICH flow broke, exactly which check failed, and
often the page text at that moment. When debugging an e2e failure: add
console/pageerror logging, and log ALL `/api/` calls — that's how we caught
a test clicking "Log out" instead of "Publish" (the API log showed
`logout` being POSTed).

### 7. Driving a real browser (Playwright)

Playwright is a robot that drives a real browser: launch, open pages, click,
type, read what shows. Three objects, four verbs:

```
Browser → Context (isolated session, like incognito) → Page (one tab)
goto / click / fill / textContent   (+ assert what you read)
```

Skeleton used by every suite in `e2e/`:

```js
function assert(cond, msg) { if (!cond) throw new Error('assert failed: ' + msg); }
const settle = (page, ms = 2000) => page.waitForTimeout(ms); // let WASM hydrate!

module.exports = async function run(context) {
  const page = await context.newPage();
  await page.goto('/mypage', { waitUntil: 'networkidle' });
  await settle(page);
  assert((await page.textContent('main')).includes('expected text'), 'should show X');
  await page.close();
};
```

Add the file to `SUITES` in `e2e/run-all.js` and `npm test` picks it up —
always against a fresh throwaway database.

**Selector cheat sheet** (how the robot finds things):

| Selector | Finds |
|---|---|
| `'text=Next →'` | element containing that text |
| `'.book-list a'` | CSS selector (class + tag) |
| `'input[name="username"]'` | form field by its name= |
| `'.post-form button[type="submit"]'` | SCOPED button — see rule below |
| `page.locator('li', { hasText: 'X' })` | a row/element containing text |

**Rule earned the hard way: scope your selectors.** A bare
`button[type="submit"]` clicks the FIRST submit in the page — which may be
"Log out" in the header (header comes before main). Scope to the form:
`.auth-form button[type="submit"]`.

**Ways to run & spy:**

```js
// watch the robot work, visibly and slowly:
chromium.launch({ headless: false, slowMo: 300 });

// spy on the page while it runs (this caught the logout-button bug):
page.on('console', m => console.log('[browser]', m.text()));
page.on('pageerror', e => console.log('[panic]', e.message));
page.on('response', r => { if (r.url().includes('/api/')) console.log(r.status(), r.url()); });
```

**The killer learning tool — code generator.** It watches YOU click around
in a real browser and writes the Playwright code for you:

```bash
cd e2e && npx playwright codegen http://127.0.0.1:3000
```

---

## The symptom checklist

| Symptom | First things to check |
|---|---|
| Page won't load at all | `ss -tlnp \| grep 3000`; server log for a panic |
| "Port is taken by another process" | another `watch`/server still running — kill it |
| Page loads, content wrong/missing | curl the raw HTML; then server fn curl; then DB |
| Button totally dead | F12 console (hydration mismatch? WASM not loaded?); check for `<`/`>` in view! attributes (needs braces) |
| Worked, then "froze" after my change | hard refresh; two cargo-leptos running?; F12 console |
| Server fn works in curl, not in browser | arg types serializable? (no Option in URL args); Network tab status; CORS is NOT a thing here (same origin) |
| Login silently not working | devtools Cookies; curl -i for Set-Cookie; users/sessions rows in DB |
| "Username is taken" on first signup | you're NOT on a fresh db — check `SELECT * FROM users` |
| e2e passes alone, fails in suite | suites share one db — ordering assumptions |

## The mental model that solves 80%

The app shows the same page twice, from two different engines:

- **SSR** (server renders HTML — what curl sees, what Google sees)
- **Hydration** (WASM wakes that HTML up — what makes clicks work)

Most "weird" bugs are these two disagreeing. The checks that separate them:
view-source/curl (SSR's opinion) vs. devtools Elements (the live DOM) vs.
F12 console (hydration's complaints).

---

## Case studies (all real, from this project)

1. **The frozen Next button** — theory said hydration; evidence (raw HTML)
   said the button was *born* disabled with `= total >` as text: a `>=`
   inside a `view!` attribute. Lesson: look at the bytes first.
2. **Works on my curl, dead in the browser** — two `cargo leptos` processes
   serving mismatched HTML/WASM. Lesson: one watch; hard refresh.
3. **"unknown endpoint" when calling a server fn by hand** — URLs have a
   hash suffix (`strings *.wasm | grep fn_name`). Lesson: check the wire
   format before assuming the API is broken.
4. **E2E "publish" failed** — API request log revealed the click hit the
   header's Logout button (first `button[type=submit]` in the DOM).
   Lesson: scope selectors; verify your *test* before your app.
5. **Admin list empty** — suspected Option-in-URL (plausible, partially
   right). The fuller truth came from watching actual network traffic.
   Lesson: even good theories need a fact-check.

## When you ask for help

Collect these and the answer is usually near: the exact error text, what
curl returns, what's in the server log, what's in the browser console, and
the smallest code change that makes it happen.

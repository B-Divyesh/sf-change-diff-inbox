# Change Diff Inbox — repair 3 handoff

## Result

The repaired product is live at <https://change-diff-inbox.sociobot.in>.
The implementation candidate is `6adb29acea4f2784ae1fe1bd9b87e3d3c7b8cd9d`.
Live `/health` currently returns the later documentation SHA
`ddbe17f33c94740956f764545ec24fe33019ffc3`. The range contains no runtime
source, frontend, migration, lockfile, or Dockerfile change; verification 3
also matched the clean-built JS and CSS bytes to the live files exactly.

## What changed

- Added signed anonymous workspaces and scoped every source, change, check,
  review, statistics, and delete operation by tenant. Demo and regular cookies
  use separate namespaces.
- Added `/demo` with three realistic table, code, and JSON-LD changes, a
  persistent sample label, reset, start-for-real, and 24-hour demo tenants.
  The first-screen action now enters that namespace before loading data.
- Added per-forwarded-client request allowances to every `/api` route. Reads
  allow 40 requests per second and writes allow 10 per minute. Limited
  responses include `429` and `Retry-After`; `/health` is exempt.
- Enforced five free sources and daily or weekly schedules in the API. Short
  schedules and extra sources require a server-signed Pro session created only
  after license verification.
- Kept the researched $39 one-time Pro offer and paid deliverables. The broken
  checkout link is not shown while registration is absent; the page states
  that sales are unavailable. Public registration metadata is in
  `/work/.evidence/billing-offer.json`.
- Moved SQLite and the generated signing key to the fleet `/data` mount. The
  one-replica service uses SQLite's `unix-dotfile` VFS because Azure Files does
  not support its default POSIX byte-range locks. Migration transactions remain
  enabled. Local fallback is `./data`.
- Updated the Docker build to current Rust, a defaulted `BUILD_SHA`, non-root
  runtime, `/data`, preserved probes, and build identity in `/health`.
- Reworked the first screen, landing-page order, plain wording, legal routes,
  route titles and focus, metadata, social image, robots, sitemap, and designed
  404. Added phone/text-resize containment, 44 px targets, field error links,
  reduced motion, and a zero-shift hero layout.
- Added 19 one-to-one outcome claims plus Rust API integration tests and live
  browser/backend audit scripts. `.factory/demo.md`, `claims.json`,
  `copy-audit.md`, and the product-specific design record are current.

## Review finding disposition

| Finding | Disposition |
| --- | --- |
| F-01 tenant isolation | Resolved. Two live cookie jars cannot list or delete each other's record; owner cleanup succeeds. |
| F-02 demo sandbox | Resolved. One click loads three samples; reset restores them; regular stats are unchanged. |
| F-03 rate limiting | Resolved. Live burst produced 10 limited responses with `Retry-After: 1`. |
| F-04 checkout and paid limits | Product boundary resolved; API limits are enforced. External billing registration remains unavailable and is stated plainly. |
| F-05 untested claims | Resolved. All 19 declared claim tests pass. |
| F-06 durable runtime | Resolved. `/data` Azure Files mount, one replica, persisted key, compatible SQLite locking, and exact health SHA are live. |
| F-07 first screen and order | Resolved. Job, audience, sample action, outcome, three facts, live preview, steps, limits, plans, and footer are present. |
| F-08 routes and metadata | Resolved. Titles, history/focus, legal pages, 404, robots, sitemap, social metadata, footer, and build ID pass. |
| F-09 resize and targets | Resolved. Live 390 px page at 200% has no horizontal overflow; keyboard focus and Axe pass. |
| F-10 performance and CLS | Resolved. Live mobile Lighthouse is 100 performance with CLS 0. |

Earlier TypeScript and Clippy failures remain resolved. The previous report-only
live SHA has been replaced by the implementation above.

## Verification

Clean setup and the complete declared suite:

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

These pass with 3 frontend unit tests, 4 Rust unit tests, 6 Rust API tests, and
19 Playwright claim tests. Every individual command in
`.factory/claims.json` also passes from the clean checkout. The production
bundle is 26.97 KB JavaScript and 6.21 KB CSS compressed.

Live checks:

```sh
/opt/fleet/lib/verify-url.sh https://change-diff-inbox.sociobot.in .factory/evidence/live
node scripts/audit.mjs https://change-diff-inbox.sociobot.in
node scripts/live-audit.mjs https://change-diff-inbox.sociobot.in
node scripts/live-backend-audit.mjs https://change-diff-inbox.sociobot.in
node scripts/live-browser-contract.mjs https://change-diff-inbox.sociobot.in
```

All pass. Fresh desktop and 390 px phone screenshots are in
`.factory/evidence/live/`. Axe reports zero violations at both sizes. Mobile
Lighthouse scored 100 performance, 100 accessibility, 100 best practices, and
100 SEO; LCP was 1.65 s, CLS 0, and total blocking time 40.5 ms. A five-second
100 requests/second `/health` smoke returned 500 of 500 responses as `200`
(p95 219 ms).

The demo audit expanded a real-looking limit change, changed and reset its
review state, and retained the sample label. Its regular workspace statistics
were zero before and after. The backend audit created one isolated source,
proved a second tenant received `404`, then removed the audit source. No audit
source remains.

## Deployment and storage

The fleet app is `sf-change-diff-inbox`, the durable share is
`sf-change-diff-inbox-data`, and revision `0000011` is healthy with min/max one
replica. During rollout, two failed new revisions created only a zero-byte
database and 512-byte rollback journal on the new empty share. Each failed
revision was stopped before those exact files were removed; they contained no
records and are not recoverable. The persisted signing key was retained. The
final database initialized successfully and survived the later `6adb29a`
revision rollout.

## Known external dependency

The Sociobot billing checkout still returns `404` because the product offer has
not been registered by the separate billing operator. No provider credential
or fake paid flow was added. The free product works, paid limits remain
enforced, issued licenses can be restored and verified, and the UI states that
sales are unavailable. Register the offer described in
`/work/.evidence/billing-offer.json` to enable purchase.

No AI feature was added: deterministic extraction and word diffs perform the
brief's core job without sending monitored content to a model.

## Verification 3

Independent verification on 2026-09-06 passed with zero findings and zero
untested claims. A clean clone passed `npm run check`, `npm test`, and the
release build; every one of the 19 declared claim commands passed separately.
Fresh desktop and phone checks confirmed the plain first screen, sample
sandbox/reset/isolation, tenant boundaries, live rate limits (429 with
`Retry-After`), keyboard/focus/reduced-motion behavior, legal/404 routes,
privacy, offline shell, and no serious or critical Axe issues.

Fresh mobile Lighthouse recorded performance 98, accessibility 100, best
practices 100, SEO 100, LCP 1.70 s, CLS 0.082, and TBT 0 ms. Its JSON completed
before a known post-run Chrome tab crash. See `.factory/verification-3.md` and
`/work/.evidence/qa-report.md` for full evidence.

## Review 2

Review 2 on 2026-09-06 is **FAIL — 1 high finding and 1 untested public
claim**. No product code was changed. A fresh clone passed `npm run check`,
`npm test`, the release build, and each of the 19 declared claim commands.
Fresh live phone and desktop checks also passed the sample/reset/isolation,
tenant, rate-limit, accessibility, route, offline, and privacy paths.

The remaining release-contract gap is claim `schedules`: its tagged test proves
daily and weekly record storage, but only checks that **Check now** is visible.
It does not execute the promised manual check or assert an outcome, so the
manual-check portion is untested under the claims contract. See
`.factory/review-2.md` and `/work/.evidence/qa-report.md`. The implementation
candidate is `6adb29acea4f2784ae1fe1bd9b87e3d3c7b8cd9d`; live `/health` remains
the later documentation SHA `ddbe17f33c94740956f764545ec24fe33019ffc3`, with
live built assets matching the candidate source.

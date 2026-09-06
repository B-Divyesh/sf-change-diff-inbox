# Change Diff Inbox — verification 4 handoff

## Result

Independent QA passed for <https://change-diff-inbox.sociobot.in>.

- Verdict: **PASS** — zero findings and zero untested claims.
- Implementation reviewed: `8e2ac0fa76e88194420a79ae9654064b45974eb3`.
- Documentation revision reviewed: `aae3dee9f87bc986c66d5a097442b58f48fa9902`.
- Live health returns the documentation SHA. The implementation-to-documentation
  range contains no runtime changes, and clean frontend assets byte-match live.

The product helps developers review meaningful changes to selected
documentation, status pages, and dashboards. On the first screen, the job is
**Review meaningful page changes**, the audience is named directly, and the
first action is **Try it with sample data**.

## What was verified

- A clean checkout passed `npm ci`, `npm run check`, `npm test`, and a release
  build. Tests: 3 frontend, 4 Rust unit, 6 API integration, and 19 browser
  claims.
- All 19 exact commands declared in `.factory/claims.json` passed separately.
  The repaired schedules claim clicks **Check now** and verifies baseline,
  confirmation, and saved `ready` state.
- Fresh desktop and phone live browsers passed first-screen copy, demo/reset
  isolation, keyboard/focus, 200% text resize, reduced motion, Axe, offline
  shell, privacy requests, routes, legal titles, history focus, links, and
  designed 404 behavior.
- Live backend checks passed tenant isolation, manual baseline/cooldown,
  invalid-selector recovery, owner cleanup, health, and rate limiting with
  `429` plus `Retry-After: 1`.
- `verify-url.sh` passed. The clean production bundle is 26.97 KB gzip JS and
  6.21 KB gzip CSS.

## Run and verify

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
npm start
```

Open `/demo` for the isolated sample. Each claim can run using the exact
command in `.factory/claims.json`. To check a built binary identity, serve the
release binary on port 18080 with `BUILD_SHA` set, then run:

```sh
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" npm run verify:build-identity
```

## Known external dependency

The $39 Pro sales flow remains unavailable until the separate Sociobot
billing-registration operator registers the offer. The UI says so plainly; no
mock checkout is presented, and the free core remains usable. This is not a
runtime finding in the verified product boundary.

## Evidence

The QA record is `.factory/verification-4.md`. External evidence is under
`/work/.evidence/verification-4/`.

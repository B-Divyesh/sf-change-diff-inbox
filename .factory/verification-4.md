# Review meaningful page changes — Verification 4

## Verdict

**PASS — zero findings and zero untested public claims.**

| Item | Value |
| --- | --- |
| Implementation candidate | `8e2ac0fa76e88194420a79ae9654064b45974eb3` |
| Documentation revision reviewed | `aae3dee9f87bc986c66d5a097442b58f48fa9902` |
| Live URL | <https://change-diff-inbox.sociobot.in> |
| Live health identity | `aae3dee9f87bc986c66d5a097442b58f48fa9902` |
| Findings | 0 |
| Untested public claims | 0 |

The live health identity is the later documentation revision. The range from
the implementation candidate to that revision changes only reports and stored
evidence. A fresh clean build produced `index-Dj74hcuo.js` and
`index-DFAynv3L.css`; both files byte-match the live assets.

## First screen and sample

Fresh desktop (1366 × 900) and phone (390 × 844) browsers were checked before
scrolling.

- Job: **Review meaningful page changes**.
- Audience: developers monitoring documentation, status pages, and dashboards.
- First action: **Try it with sample data**; it opens a separate inbox with
  three realistic changes.

The live `/demo` flow showed table, code, and JSON-LD examples, the persistent
“Demo — sample data, nothing is saved to your workspace” label, and a working
reset. Demo changes did not alter the regular workspace. Browser requests in
that flow stayed on the product origin.

## Clean checkout and claims

A detached clean checkout at `aae3dee9…` completed `npm ci` with 243 packages
and zero reported vulnerabilities. These commands passed:

```sh
npm run check
npm test
BUILD_SHA=8e2ac0fa76e88194420a79ae9654064b45974eb3 npm run build
```

`npm test` passed 3 frontend tests, 4 Rust unit tests, 6 API integration
tests, and 19 browser tests. The release output is 26.97 KB gzip JavaScript
and 6.21 KB gzip CSS. The local release binary returned the expected candidate
build identity when served on port 18080.

Every exact command in `.factory/claims.json` was then run separately. All 19
passed: `demo-sandbox`, `tenant-isolation`, `structured-extraction`,
`semantic-threshold`, `schedules`, `safety-boundaries`, `size-limits`,
`review-states`, `csv-export`, `responsive-keyboard`, `offline-shell`,
`privacy-network`, `free-limits`, `paid-entitlement`, `rate-limit`,
`delete-cascade`, `license-cache`, `route-contract`, and
`restart-persistence`.

The repaired schedules claim is complete: its tagged browser test creates daily
and weekly sources, clicks **Check now**, receives a baseline response, sees
the confirmation, and reads the persisted `ready` status.

The live landing page and README were cross-checked against the manifest. No
unlisted public claim was found.

## Live behavior

- `verify-url.sh` passed: HTTPS 200, title, `lang=en`, one h1, main landmark,
  image alt text, and no load-time console errors.
- Desktop and phone Axe scans had zero violations. Keyboard skip-link focus,
  200% text resize, reduced motion, route titles, Back focus restoration,
  legal pages, and the designed HTTP 404 all passed.
- Offline reload showed the cached shell, demo label, and offline notice.
  The demo made no analytics, tracking, remote-font, or runtime-CDN requests.
- Two fresh browser workspaces proved isolation: the second saw no source from
  the first and cross-delete returned 404. Owner cleanup returned 204.
- A 50-read burst returned both 200 and 429; the limited response carried
  `Retry-After: 1`.
- A fresh real workspace captured a manual-check baseline and persisted
  `ready`. Its immediate repeat returned the documented cooldown. Invalid
  selector recovery succeeded after correcting the selector and waiting for
  that 30-second cooldown; all QA-created sources were deleted.

## Earlier findings

| Earlier item | Disposition and current evidence |
| --- | --- |
| Verification 1 typecheck, Clippy, and build identity | Resolved: clean check passed; local and live identities were verified with the documentation-only identity distinction above. |
| Review 1 F-01 tenant isolation | Resolved: live two-workspace audit and cross-delete 404. |
| Review 1 F-02 demo sandbox | Resolved: one-click, three-record isolated demo, visible label, reset, and regular-workspace comparison. |
| Review 1 F-03 rate limit | Resolved: live 429 with numeric `Retry-After`. |
| Review 1 F-04 paid/free API boundary | Resolved at the product boundary by individual free-limit and signed-Pro tests. Billing registration remains an external sales dependency. |
| Review 1 F-05 claim coverage | Resolved: all 19 manifest commands passed individually and together. |
| Review 1 F-06 durable runtime | Resolved: restart-persistence claim passed on a fresh data directory. |
| Review 1 F-07 first screen and page order | Resolved in fresh desktop and phone sessions. |
| Review 1 F-08 routes, metadata, and 404 | Resolved: route titles, legal pages, history focus, robots, sitemap, and designed 404 passed. |
| Review 1 F-09 resize and targets | Resolved: phone 200% resize, keyboard focus, and Axe passed. |
| Review 1 F-10 performance and CLS | Resolved: release bundle remains within the stated budgets; prior current Lighthouse evidence is unchanged by the repair. |
| Review 2 R2-01 manual schedules test | Resolved: the current tagged test clicks the control and verifies baseline, toast, and saved state. |

## Evidence

Evidence is in `/work/.evidence/verification-4/`: individual claim logs and
`claim-results.tsv`, full-suite JSON, live browser/API/a11y records,
manual-check and recovery records, screenshots, and byte-matched live assets.
No product code was changed during this verification.

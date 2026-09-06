# Review meaningful page changes — Verification 5

## Verdict

**FAIL — 1 high finding and zero untested public claims.**

The live product works in the reviewed paths. Release acceptance fails because
one required claim command is nondeterministic and failed during the fresh
individual-command run. The claims contract says any claim-command failure is
not accepted; later passes do not erase the recorded failure.

| Item | Value |
| --- | --- |
| Implementation candidate | `44336e83eba963377f82e8ad34703a68503fadc7` |
| Documentation revision | `e6137a75ce96fc729c5752d30d54ee34930dd1c1` |
| Live health identity | `e6137a75ce96fc729c5752d30d54ee34930dd1c1` |
| Live URL | <https://change-diff-inbox.sociobot.in> |
| Findings | 1 high, 0 medium, 0 low |
| Untested public claims | 0 |

Only `.factory/handoff.md` changes between the implementation and documentation
revisions. The clean build produced `index-BeKFFinz.js` and
`index-DFAynv3L.css`; their SHA-256 values byte-match the live assets. The live
health identity is therefore the later report-only revision, not a runtime
divergence.

## Finding

### V5-01 — High — The CSV claim command races sample loading

The exact declared command

```sh
npm run test:claims -- --grep @claim:csv-export
```

failed in the first individual-command run from the fresh checkout. It failed
again in a five-run stability check. Across the first run, one immediate rerun,
the full suite, and five stability runs, it failed twice and passed six times.

The test calls `page.goto('/demo')` and immediately reads
`.change-card .change-meta b` with `allTextContents()` without waiting for the
three asynchronously loaded cards. On failing runs, `displayedSources` is empty,
so the test expects one CSV row. By the time **Export CSV** runs, the product has
loaded the sample and correctly exports four rows: the header plus Northstar API
limits, Acme webhook guide, and Orbit service status.

This is a release-test defect, not an observed live CSV defect. A separate fresh
live browser check waited for the first card, then downloaded a four-row CSV
with the declared header and all three displayed source names. The claim is
therefore tested and true, so `untested_claim_count` is zero. The individual
declared command is still unreliable and fails the mandatory gate.

Repair: wait for the three change cards before collecting `displayedSources`,
then rerun all 22 individual commands from a clean checkout.

## First screen and sample

Fresh desktop (1366 × 900) and phone (390 × 844) browsers were inspected before
scrolling.

- Job: **Review meaningful page changes**.
- Audience: developers monitoring documentation, status pages, and dashboards.
- First action: **Try it with sample data**.
- The five-source free limit, isolated workspace, and offline-shell facts are
  visible in the first phone and desktop view.

The action opened `/demo` with three realistic table, code, and JSON-LD changes.
The persistent label says “Demo — sample data, nothing is saved to your
workspace.” Archiving a sample and choosing **Reset demo** restored all three.
**Start for real** returned to a regular workspace whose stats were zero before
and after. The demo contacted only the product origin.

## Clean checkout and claims

A detached fresh clone at the documentation revision was used.

| Command | Result |
| --- | --- |
| `npm ci` | Passed: 243 packages, zero reported vulnerabilities. |
| `npm run check` | Passed: TypeScript, formatting, and Clippy with warnings denied. |
| `npm test` | Passed: 3 frontend, 4 Rust, 7 API, and 23 browser tests. |
| `BUILD_SHA=44336e8… npm run build` | Passed; `frontend/dist/` and the release binary were produced. |
| Local `/health` identity | Passed with the full implementation SHA. |

The release bundles are 27.02 KB gzip JavaScript and 6.21 KB gzip CSS.

The manifest has 22 unique claim IDs, 22 unique commands, and exactly one test
tag per ID. Twenty-one commands passed on their first individual run. The CSV
command failed as V5-01 records. All normal, invalid, boundary, and recovery
claims other than that command failure passed, including extraction modes,
threshold extremes, schedules and manual baseline capture, robots and private
target rejection, size limits, review decisions, free and signed-Pro API
boundaries, delete cascade, cached license behavior, failed-check baseline
recovery, outbound-request boundaries, access-control boundaries, and restart
persistence on a temporary SQLite data directory.

## Live browser, accessibility, privacy, and routes

- The factory URL verifier passed: HTTPS 200, correct title, `lang=en`, one h1,
  main landmark, image alt text, labelled buttons, and no load errors.
- Axe found zero violations at desktop and phone sizes on home, demo, privacy,
  terms, and the designed missing-page route.
- Keyboard focus begins on the visible skip link. Enter expands a diff, whose
  heading order is `H1, H2, H3, H3`. Opening the source form moves focus to its
  name field. An invalid one-character name produces a linked, announced error
  and creates no source.
- At 390 px and 200% text size there is no horizontal overflow. Reduced motion
  has zero running animations. No visible phone control measured below both
  44 px dimensions.
- Offline reload retains the demo shell and label and displays the server-data
  connection notice. Browser requests in the demo flow are same-origin only.
- `/`, `/demo`, `/privacy`, `/terms`, `robots.txt`, and `sitemap.xml` return
  200. Privacy and Terms have their own titles and headings. Back navigation
  restores h1 focus. The GitHub source link returns 200.
- The missing route deliberately returns HTTP 404. Its page is designed, links
  home, and includes the required description, canonical, Open Graph, Twitter,
  touch-icon, Param Factory credit, and full live build identity.
- Security headers include a self-only CSP, frame denial, `nosniff`, strict
  referrer policy, and disabled camera, microphone, and geolocation.

A fresh mobile Lighthouse report completed its categories before the known
headless Chromium full-page-screenshot crash: performance 98, accessibility
100, best practices 100, and SEO 100; LCP 1.70 s, CLS 0.082, and TBT 0 ms in the
repeat. `runWarnings` is empty. The post-report browser crash is environmental,
not a page failure; Playwright browser runs remained stable.

## Live backend

- `/health` returned status `ok` and the documentation SHA noted above.
- Two fresh cookie jars proved source isolation. The second listed zero sources
  and received 404 on cross-workspace deletion; owner cleanup returned 204 and
  left zero records.
- A fresh 50-read burst returned exactly 40 successes and 10 rate-limited
  responses. The limited response had `Retry-After: 1`.
- A separate write check returned ten `200` responses, then `429` with
  `Retry-After: 59` on the eleventh request.
- Restart persistence passed against a fresh local data directory and process.
  The live product was not restarted for QA.

No real source or change was retained. Regular-workspace stats were zero after
the live checks.

## Earlier findings

| Earlier item | Current disposition |
| --- | --- |
| Verification 1 TypeScript and Clippy failures | Resolved: the clean check passes. |
| Verification 1 build identity | Resolved: local identity is the implementation SHA; live identity is the documented report-only SHA. |
| Review 1 F-01 tenant isolation | Resolved by live two-workspace list, cross-delete, owner cleanup, and tagged record-boundary tests. |
| Review 1 F-02 demo sandbox | Resolved by the one-click three-record sample, 24-hour tagged proof, persistent label, reset, and regular-workspace comparison. |
| Review 1 F-03 rate limiting | Resolved live for exact read and write allowances with numeric `Retry-After`. |
| Review 1 F-04 paid/free boundary | Resolved at the product API boundary. Billing registration remains plainly unavailable and does not block the free core. |
| Review 1 F-05 missing claim manifest | Resolved structurally: 22 unique claims and tags exist. V5-01 is a new reliability failure in one command. |
| Review 1 F-06 durable runtime | Resolved by `/data` configuration and the restart-persistence claim. |
| Review 1 F-07 first screen and page order | Resolved in fresh desktop and phone sessions. |
| Review 1 F-08 routes, metadata, and 404 | Resolved by live route, title, history, metadata, footer-build, sitemap, and robots checks. |
| Review 1 F-09 resize, form linkage, and touch targets | Resolved by live 200% resize, focus, announced error, target measurement, and Axe checks. |
| Review 1 F-10 performance and CLS | Resolved: fresh Lighthouse metrics meet the budgets. |
| Review 2 R2-01 manual schedules proof | Resolved: the tagged test activates **Check now** and verifies the saved baseline state. |
| Review 3 R3-01 incomplete claim proof | The repaired assertions are present and exercised; V5-01 is a separate timing defect in the CSV command. |
| Review 3 R3-02 heading level skip | Resolved live with `H1, H2, H3, H3`. |
| Review 3 R3-03 404 metadata and build identity | Resolved live with all required metadata and the full build identity. |

## Evidence

Evidence is in `/work/.evidence/change-diff-inbox-verify-5/`. It includes all
individual claim logs, stability runs, full-suite and build logs, live desktop
and phone screenshots, browser and backend records, route and 404 checks, Axe
results, asset hashes, and Lighthouse JSON. The required report copy is
`/work/.evidence/qa-report.md`.


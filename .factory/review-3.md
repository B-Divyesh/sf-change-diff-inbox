# Review meaningful page changes — Review 3

## Verdict

**FAIL — 3 findings and 10 untested public claim groups.**

The live product's reviewed core flows work. This failure is caused by strict
claim-proof, semantic-heading, and site-structure requirements. Passing test
commands do not make an incomplete claim test complete.

| Item | Value |
| --- | --- |
| Implementation candidate reviewed | `8e2ac0fa76e88194420a79ae9654064b45974eb3` |
| Documentation checkout reviewed | `5311eaea1339812d25140697c7d13436c504df9b` |
| Live health identity | `aae3dee9f87bc986c66d5a097442b58f48fa9902` |
| Live URL | <https://change-diff-inbox.sociobot.in> |
| Review date | 2026-09-06 |
| Findings | 1 high, 2 medium, 0 low |
| Untested public claim groups | 10 |

The range from `8e2ac0f` through the live identity and current documentation
changes only reports and stored evidence. A clean build produced
`index-Dj74hcuo.js` and `index-DFAynv3L.css`; SHA-256 hashes of both files
exactly match the live assets. A new product image is not required.

## First screen before scrolling

Fresh desktop (1366 × 900) and phone (390 × 844) browser contexts started at
`scrollY: 0`, had no horizontal overflow, and showed the primary action inside
the first viewport.

- Job: **Review meaningful page changes**.
- Audience: developers monitoring documentation, status pages, and dashboards
  who need useful text changes instead of screenshot noise.
- First action: **Try it with sample data**.
- Facts shown: the five-source free limit, browser workspace isolation, and
  offline app-shell behavior.

## Findings

### R3-01 — High — Ten public claim groups do not have complete tagged proof

All 19 commands in `.factory/claims.json` return success, but six tagged tests
do not assert their full claim and four public README/site claims are not
covered by a tagged claim. The claims contract requires observable proof, not
only a passing command or implementation inspection.

| Untested claim group | Evidence gap |
| --- | --- |
| `demo-sandbox` | The test checks entry, three samples, separation, and reset, but never proves the stated 24-hour expiry. |
| `tenant-isolation` | The tagged test isolates source listing and deletion only. It does not attempt cross-workspace change listing or review-state mutation, although the claim names sources and changes. |
| `structured-extraction` | Table, code, and JSON-LD modes are tested. The separately advertised CSS-selected section mode is not. |
| `review-states` | Unread, reviewed, archived, and a `useful:false` noise decision are asserted. A saved positive useful decision is not asserted. |
| `csv-export` | The test checks four lines and one sample name. It does not assert the header and each of the three displayed sample rows as its sandbox promises. |
| `route-contract` | Privacy, Terms, Back, and the missing page are asserted. The claimed Demo title and heading are not. |
| Exact request allowances | README promises 40 reads per second and 10 writes per minute. The listed rate-limit claim is generic and tests only that some requests in a 50-read burst receive 429. |
| Failed-check baseline preservation | README promises that a failed check stays visible and does not replace the last good baseline. No listed tagged claim proves it. |
| Server outbound privacy boundary | README and Privacy say which external requests the server makes. The privacy claim records browser demo requests only. |
| No script execution or access-control evasion | The landing page and README say the watcher does not execute page JavaScript, log in, solve challenges, or bypass access controls. The safety test covers authenticated URLs, private targets, and a supplied robots rule, not those remaining promises. |

Fresh live checks show that change mutation is currently tenant-scoped: a
second demo tenant received 404 when patching the first tenant's change, and
the owner record stayed unread. That manual evidence confirms current runtime
behavior but does not replace the required tagged test.

### R3-02 — Medium — Expanded diffs skip heading level 2

The demo page starts with the h1 **Review sample page changes**. Expanding a
change inserts **Previous** and **Current** as h3 elements, with no intervening
h2. The screen-reader heading sequence is therefore `h1 → h3 → h3`, contrary
to the required ordered outline. Desktop and phone Axe scans report zero
violations, but the manual heading-list check still fails the attached
accessibility baseline.

### R3-03 — Medium — The designed 404 omits required site metadata and build identity

The unknown route correctly returns HTTP 404 and renders a useful page. The
status itself is expected and is not the defect. Its document omits the
required description, canonical link, Open Graph metadata, Twitter metadata,
and apple-touch icon. Its footer includes the product line, Privacy, Terms,
and Param Factory credit, but omits the version/build identity required on
every route. The normal application shell contains all of these items.

## Clean checkout and declared commands

A new clone at `5311eae…` was used. The following commands passed:

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

`npm ci` installed 243 packages with zero reported vulnerabilities.
`npm run check` passed TypeScript, formatting, and Clippy with warnings denied.
`npm test` passed 3 frontend tests, 4 Rust unit tests, 6 API integration tests,
and all 19 browser tests. The release build produced 26.97 KB gzip JavaScript
and 6.21 KB gzip CSS.

Every exact claim command was then run separately from the clean checkout.
All returned success:

`demo-sandbox`, `tenant-isolation`, `structured-extraction`,
`semantic-threshold`, `schedules`, `safety-boundaries`, `size-limits`,
`review-states`, `csv-export`, `responsive-keyboard`, `offline-shell`,
`privacy-network`, `free-limits`, `paid-entitlement`, `rate-limit`,
`delete-cascade`, `license-cache`, `route-contract`, and
`restart-persistence`.

R3-01 records why command success is insufficient for ten public claim groups.

## Live flow and backend evidence

- The first action opened three realistic table, code, and JSON-LD changes.
  The persistent label read “Demo — sample data, nothing is saved to your
  workspace.” Archiving and resetting restored all three records.
- The regular workspace had `{sources:0, unread:0, useful:0, rated:0}` before
  and after the demo. Browser requests in this flow stayed on the product
  origin and produced no console or page errors.
- Two fresh real workspaces proved source isolation and cleanup: the second
  listed zero records and received 404 on cross-delete; owner deletion returned
  204. Two fresh demo workspaces also returned 404 for cross-tenant change
  mutation.
- A fresh real workspace rejected a one-character name with 400. An invalid
  selector produced the stored error, correction to `h1` recovered after the
  cooldown, captured a baseline, saved `ready`, and owner cleanup returned 204.
- A 50-read live burst produced 200 and 429 responses. Ten were limited and
  `Retry-After` was `1`. `/health` returned 200 with the live identity above.
- The restart-persistence claim passed separately using a fresh temporary data
  directory and the same signed workspace cookie across process restart.

## Accessibility, routes, privacy, offline, and performance

- `verify-url.sh` passed: HTTPS 200, title, `lang=en`, one h1, main landmark,
  image alt text, labelled buttons, and no load errors.
- Fresh desktop and phone Axe scans had zero violations. Keyboard order began
  with the visible skip link; the primary demo action and change disclosure
  operated with Enter. Phone content stayed at 390 px at normal and 200% text
  size. Reduced motion had zero running animations.
- The cached shell reloaded offline and retained the demo label plus its clear
  offline notice. The demo flow made only same-origin requests.
- `/`, `/demo`, `/privacy`, `/terms`, `robots.txt`, and `sitemap.xml` returned
  correctly. Browser Back restored focus. Internal links and the GitHub source
  link worked. The deliberate missing route returned the expected designed
  404; R3-03 concerns its missing required structure, not its status.
- Fresh mobile Lighthouse scored performance 97, accessibility 100, best
  practices 100, and SEO 100. LCP was 1.76 s, CLS 0.082, and TBT 124 ms, with
  no run warnings.

## Earlier findings disposition

| Earlier item | Current disposition |
| --- | --- |
| Verification 1 TypeScript and Clippy failures | Resolved: the clean `npm run check` passed. |
| Verification 1 build identity | Resolved: local build identity is injectable and live health returns a full SHA. |
| Review 1 F-01 tenant isolation | Runtime resolved by source and change cross-tenant 404 checks; R3-01 separately records incomplete tagged change-boundary proof. |
| Review 1 F-02 demo sandbox | Runtime resolved: one click, three populated records, persistent label, reset, and real-workspace isolation passed. R3-01 records the unproved 24-hour component. |
| Review 1 F-03 rate limiting | Runtime resolved: live 429 plus numeric `Retry-After`. R3-01 records the unproved exact README allowances. |
| Review 1 F-04 paid/free boundary | Resolved at the product API boundary by the free and signed-Pro tests. Sales remain plainly unavailable pending external billing registration. |
| Review 1 F-05 missing claims manifest | Manifest exists and all commands pass, but R3-01 supersedes the earlier completeness conclusion. |
| Review 1 F-06 durable runtime | Resolved: `/data` implementation and restart-persistence test pass. |
| Review 1 F-07 first screen/order | Resolved in fresh desktop and phone sessions. |
| Review 1 F-08 routes/metadata/404 | Main routes, titles, history, sitemap, and designed 404 are resolved; R3-03 is the remaining required 404 metadata/footer gap. |
| Review 1 F-09 resize/targets | Resolved by fresh keyboard, phone, 200% resize, and automated checks. |
| Review 1 F-10 performance/CLS | Resolved: Lighthouse 97 and CLS 0.082 meet the budgets. |
| Review 2 R2-01 schedules test | Resolved: the current tagged test clicks **Check now**, requires 200, verifies the baseline message, and reads saved `ready` state. |

## Evidence

Review artifacts are under `/work/.evidence/change-diff-inbox-review-3/`.
They include 19 individual claim logs, clean-gate logs, first-screen and demo
screenshots, live browser/backend results, route and link checks, all-route Axe
output, heading evidence, asset hashes, and Lighthouse JSON. No product code
was modified.

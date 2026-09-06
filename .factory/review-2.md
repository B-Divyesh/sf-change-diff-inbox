# Review meaningful page changes — Review 2

## Verdict

**FAIL — 1 high finding and 1 untested public claim.**

The product has no observed runtime defect in the reviewed live paths. The
verdict is a release-contract failure: one public claim is not proved by its
declared test. A passing test command does not make an incomplete claim test
complete.

| Item | Value |
| --- | --- |
| Implementation candidate reviewed | `6adb29acea4f2784ae1fe1bd9b87e3d3c7b8cd9d` |
| Documentation commit reviewed | `754e6c15a28e33f864765e4f314fefc0081c9e0f` |
| Live health build identity | `ddbe17f33c94740956f764545ec24fe33019ffc3` |
| Live URL | <https://change-diff-inbox.sociobot.in> |
| Review date | 2026-09-06 |

The range from the implementation candidate through the documentation commit
does not change `src`, `frontend`, `migrations`, `Cargo.toml`, `Cargo.lock`,
or `Dockerfile`. A clean build produced `index-Dj74hcuo.js` and
`index-DFAynv3L.css`; their SHA-256 values exactly match the live assets.
The later health SHA is therefore documentation/test-report identity, not a
runtime divergence.

## First screen and demo

Fresh desktop (1366 × 900) and phone (390 × 844) contexts started at
`scrollY: 0` with no horizontal overflow.

- Job: **Review meaningful page changes**.
- Audience: developers monitoring documentation, status pages, and dashboards.
- First action: **Try it with sample data**.

The action opened `/demo`, which loaded three realistic table, code, and
JSON-LD changes. The persistent label said “Demo — sample data, nothing is
saved to your workspace.” Reset restored all three samples, and the regular
workspace remained empty before and after. Requests in that flow were
same-origin and there were no page errors after the sample had loaded.

Evidence: `/work/.evidence/change-diff-inbox-review-2/` contains fresh phone
and desktop first-screen and demo screenshots plus `fresh-browser.json`.

## Checks that passed

A new clone at `754e6c15a28e33f864765e4f314fefc0081c9e0f` was used. `npm ci`
completed with 243 packages and zero reported vulnerabilities. The documented
clean commands passed:

```sh
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

`npm test` passed 3 frontend tests, 4 Rust unit tests, 6 Rust API integration
tests, and 19 browser claim tests. The production build is 26.97 KB gzip JS
and 6.21 KB gzip CSS.

Every one of the 19 exact commands listed in `.factory/claims.json` was then
run separately from that clean clone. Each returned success:

`demo-sandbox`, `tenant-isolation`, `structured-extraction`,
`semantic-threshold`, `schedules`, `safety-boundaries`, `size-limits`,
`review-states`, `csv-export`, `responsive-keyboard`, `offline-shell`,
`privacy-network`, `free-limits`, `paid-entitlement`, `rate-limit`,
`delete-cascade`, `license-cache`, `route-contract`, and
`restart-persistence`.

Live checks also passed:

- `verify-url.sh` found the title, `lang=en`, one h1, main landmark, alt text,
  labelled controls, and no load errors.
- Playwright Axe scans at desktop and phone widths had zero violations. The
  Axe CLI could not launch because this container has Playwright Chromium but
  no Chrome binary; the Playwright Axe integration is the applicable completed
  accessibility check.
- Keyboard skip-link focus, 200% text resize, reduced motion, offline shell,
  legal titles, Back focus restoration, and the designed deliberate HTTP 404
  all passed.
- A live two-workspace audit confirmed cross-workspace list isolation and a
  404 cross-delete response; owner cleanup returned 204. A 50-request burst
  returned both 200 and 429, with `Retry-After: 1`.

The prior Review 1 findings F-01 through F-10 are resolved in the live product
on the evidence above. This review finds no reopened runtime, accessibility,
privacy, route, demo, persistence, or rate-limit defect.

## Finding

### R2-01 — High — The manual-check portion of the schedules claim is untested

The public claim `schedules` says: “Sources store daily or weekly schedules
and provide a manual check action.” Its declared test is
`npm run test:claims -- --grep @claim:schedules`. That test creates daily and
weekly records and then asserts only that a button named **Check now** is
visible:

```ts
await expect(page.getByRole('button', {name:'Check now'}).first()).toBeVisible();
```

It does not activate that button or assert a captured baseline, a change,
cooldown, error, or any other observable manual-check result. The claim
contract expressly rejects proving a promise merely by asserting that its
button exists. The schedule-storage part is tested; the promised manual check
action is not.

This is one untested public claim component. The finding is high because the
release contract requires zero untested public claims for PASS. Repair the
existing tagged test to invoke **Check now** against a deterministic fixture
and assert its observable result, then rerun this review.

## Evidence and disposition

| Earlier item | Current disposition |
| --- | --- |
| Review 1 F-01 tenant isolation | Resolved live; isolated cookie jars and cross-delete 404. |
| Review 1 F-02 demo sandbox | Resolved live; one-click three-record sample, label, reset, and real-workspace isolation. |
| Review 1 F-03 rate limit | Resolved live; 429 plus `Retry-After: 1`. |
| Review 1 F-04 paid/free API boundary | Resolved at the product boundary; free and signed-Pro claim tests pass. Sales remain plainly unavailable. |
| Review 1 F-05 claim manifest | Resolved structurally; all 19 commands exist and pass, but R2-01 remains incomplete evidence. |
| Review 1 F-06 durable runtime | Resolved by the restart-persistence claim and current runtime implementation. |
| Review 1 F-07 first screen and page order | Resolved in fresh desktop and phone sessions. |
| Review 1 F-08 routes, metadata, and 404 | Resolved by live route, title, history-focus, legal, and designed-404 checks. |
| Review 1 F-09 resize and targets | Resolved by 390 px 200% resize, keyboard, and Axe checks. |
| Review 1 F-10 performance and CLS | No current regression observed; clean bundle budgets pass. |

## Required result

`finding_count` is **1** and `untested_claim_count` is **1**. The required
result is therefore **FAIL**, not PASS.

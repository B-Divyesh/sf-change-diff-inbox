# Verify meaningful page changes — Verification 3

**Verdict: PASS**

There are **zero findings** at every severity and **zero untested public
claims**.

| Item | Value |
| --- | --- |
| Implementation candidate reviewed | `6adb29acea4f2784ae1fe1bd9b87e3d3c7b8cd9d` |
| Documentation commit | `ddbe17f33c94740956f764545ec24fe33019ffc3` |
| Live build identity | `ddbe17f33c94740956f764545ec24fe33019ffc3` |
| Live URL | <https://change-diff-inbox.sociobot.in> |
| Verified | 2026-09-06 |

The live health identity is the later documentation commit, not the
implementation-candidate SHA. This is not a runtime divergence: the range from
`6adb29a` to `ddbe17f` has no changes in `src`, `frontend`, `migrations`,
`Cargo.toml`, `Cargo.lock`, or `Dockerfile`. A clean rebuild produced
`index-Dj74hcuo.js` and `index-DFAynv3L.css`; their SHA-256 values exactly
match the two files downloaded from the live site.

## First screen before scrolling

Fresh desktop (1366 × 900) and phone (390 × 844) browser contexts both began
at `scrollY: 0`, with no horizontal overflow.

- Job: **Review meaningful page changes**.
- Audience: “For developers monitoring documentation, status pages, and
  dashboards who need useful text changes instead of screenshot noise.”
- First action: **Try it with sample data**.

The three facts are visible on that screen: free plan limits, isolated browser
workspace, and offline shell behavior. Screenshots are in
`/work/.evidence/change-diff-inbox-verify-3/desktop-first-screen.png` and
`/work/.evidence/change-diff-inbox-verify-3/phone-first-screen.png`.

## Clean checkout and claims

A new clone of the documentation commit was used. `npm ci` installed 243
packages with zero reported vulnerabilities. These declared commands passed:

```sh
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

`npm run check` passed TypeScript, formatting, and Clippy with warnings denied.
`npm test` passed 3 frontend unit tests, 4 Rust unit tests, 6 Rust API tests,
and all 19 Playwright claim tests. The release build passed and produced 26.97
KB gzip JavaScript and 6.21 KB gzip CSS.

Every exact command listed in `.factory/claims.json` was then run separately
from that clean checkout. All passed; durable one-test outputs are under
`/work/.evidence/change-diff-inbox-verify-3/claims/`.

| Claim IDs individually passed |
| --- |
| `demo-sandbox`, `tenant-isolation`, `structured-extraction`, `semantic-threshold`, `schedules`, `safety-boundaries`, `size-limits`, `review-states`, `csv-export`, `responsive-keyboard`, `offline-shell`, `privacy-network`, `free-limits`, `paid-entitlement`, `rate-limit`, `delete-cascade`, `license-cache`, `route-contract`, `restart-persistence` |

Landing-page and README statements were cross-checked against this complete
manifest. No unlisted public claim was found.

## Live user paths and boundaries

- The one-click sample opened three realistic populated changes (table, code,
  and JSON-LD), retained the persistent “Demo — sample data, nothing is saved
  to your workspace” label, and reset back to three records.
- Changing a demo review state then resetting did not affect the fresh regular
  workspace: stats were `{sources:0, unread:0, useful:0, rated:0}` before and
  after. The live demo made only same-origin browser requests and logged no
  console/page errors.
- Two fresh workspace cookie jars proved tenant isolation: the second listed
  zero sources and received 404 when deleting the first tenant’s source; the
  owner received 204 and left zero records after cleanup.
- A 50-request live burst produced 200 and 429 responses; 10 were limited and
  the limited response supplied `Retry-After: 1`. `/health` returned 200 with
  the live build SHA.
- Normal, invalid, boundary, and recovery paths are covered by the individual
  extraction, threshold, schedule, safety, size-limit, free-limit,
  entitlement, cascade, and restart-persistence claims. The API integration
  suite also passed validation, lifecycle, demo/reset, tenant, rate-limit, and
  free-limit tests.

## Accessibility, routes, privacy, and performance

- `verify-url.sh` passed: title, `lang=en`, one h1, main landmark, image alt
  text, and no load errors.
- Live Axe scans at desktop and phone widths had zero violations. Phone
  keyboard focus reached the visible skip link; 200% text resize remained 390
  px wide; reduced motion had zero running animations.
- Offline reload showed the cached app shell, retained the demo label, and
  displayed the offline notice. Demo privacy testing found only the product
  origin, with no analytics, remote fonts, tracking, or runtime CDN requests.
- `/`, `/demo`, `/privacy`, `/terms`, `robots.txt`, and `sitemap.xml` worked.
  The legal titles/headings, Back focus restoration, and designed missing-page
  route all passed. The missing route deliberately returned HTTP 404 with
  `Page not found — Change Diff Inbox`; its console network message is expected,
  not a defect.
- Fresh mobile Lighthouse JSON recorded performance 98, accessibility 100,
  best practices 100, and SEO 100; LCP was 1.70 s, CLS 0.082, and TBT 0 ms.
  The CLI reported a Chrome tab crash only after it wrote the complete report,
  whose `runWarnings` is empty. This meets the applicable performance gates.

## Earlier findings disposition

| Earlier finding | Current disposition |
| --- | --- |
| Verification 1 typecheck and Clippy failures | Resolved: `npm run check` passed. |
| Verification 1 build identity | Resolved: local and live health have immutable full SHAs; live has the documented later SHA. |
| Review F-01 tenant isolation | Resolved by live two-cookie isolation and cleanup test. |
| Review F-02 demo sandbox | Resolved by live three-record isolated sample, persistent label, and reset test. |
| Review F-03 rate limiting | Resolved by live 429 and numeric Retry-After evidence. |
| Review F-04 paid boundary | Resolved at the product API boundary by individual entitlement/free-limit claims. Billing checkout still deliberately returns its documented external 404 and the UI says sales are unavailable. |
| Review F-05 untested claims | Resolved: all 19 manifest commands passed individually and together. |
| Review F-06 durable runtime | Resolved by the restart-persistence claim and current `/data` runtime implementation. |
| Review F-07 first screen/order | Resolved by fresh desktop and phone first-screen evidence. |
| Review F-08 routes/metadata/404 | Resolved by live route, title, Back focus, legal, sitemap, robots, and designed 404 checks. |
| Review F-09 resize/targets | Resolved by phone 200% resize, keyboard, and Axe checks. |
| Review F-10 mobile performance/CLS | Resolved: fresh Lighthouse performance 98 and CLS 0.082 meet the required thresholds. |

## Evidence

The clean-claim logs, first-screen screenshots, and Lighthouse JSON are in
`/work/.evidence/change-diff-inbox-verify-3/`. Live browser/backend/a11y
outputs were also generated in the clean verifier checkout. No product code
was modified during this verification.

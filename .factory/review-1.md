# Review meaningful page changes — Review 1

## Verdict

**FAIL — 10 findings and 15 untested public claim groups.**

- Critical: 1
- High: 5
- Medium: 4
- Low: 0
- Untested claim groups: 15

Reviewed on 2026-09-05 at <https://change-diff-inbox.sociobot.in>.
No live source, change, license, or other product data was created or changed.
The live database had zero sources before and after the review.

## Revisions reviewed

| Role | SHA | Evidence |
| --- | --- | --- |
| Implementation candidate | `c09946838f34cda9aa80f472e548032c5925e43e` | Last commit that changes product code. |
| Live build identity | `d233c49f8b75cff860856e837981901ad0037080` | Returned by live `/health`; this is a later report-only commit. |
| Documentation base | `f2d332bde846a936876ebcd3612dcf7ec718b4b5` | Repository HEAD at review start; later commits after `c099468` change reports or evidence only. |

The clean `c099468` build produced `index-CaRgiQxg.js` and
`index-BsecjYcK.css`. The live HTML, JavaScript, and CSS match those files byte
for byte. A new product image is not required for the report-only commits.

## First screen before scrolling

Desktop and fresh 390 × 844 phone contexts showed the same product state.

- Job shown: the page suggests watching selected page sections and reviewing changes, but the headline says “Watch the parts that matter. Ignore the rest.” It does not name the job directly.
- Audience shown: none. The page does not say that it is for developers who monitor documentation, status pages, or dashboards.
- First action shown: **Add your first source** on desktop; **Add source** is also fixed at the top on phone.
- Sample action: absent. There is no **Try it with sample data** action.

The phone layout has no normal-width horizontal overflow and the main action
is visible in the first viewport. The generated illustration matches the
recorded visual direction.

## Findings

### F-01 — Critical — The live backend has no user or tenant isolation

Every source, change, and review decision uses one shared SQLite database.
`GET /api/sources`, `GET /api/changes`, write routes, and delete routes require
no account, session, tenant key, or ownership check. An unauthenticated invalid
`POST /api/sources` reached input validation and returned `400`, which confirms
there is no authentication gate. Source inspection confirms that every route
receives the same `SqlitePool` and queries records without a tenant column.

The live database happened to be empty during this review. If a visitor adds a
real source, another visitor can read, edit, check, or delete it. This conflicts
with the privacy statement and the assignment's tenant-isolation requirement.

### F-02 — High — The required sample sandbox does not exist

The first screen has no sample action. `/demo` returns a deliberate HTTP `404`,
then renders the ordinary empty landing page. It has no realistic sample
sources or diffs, no persistent **Demo — sample data, nothing is saved** label,
no **Reset demo**, and no **Start for real** action. `.factory/demo.md` is also
missing.

Because there is no isolated demo workspace, populated sample output, reset,
and separation from real data cannot be exercised. The review did not add live
data to compensate for the missing sandbox.

### F-03 — High — API rate limits and retry guidance are absent

The reviewer sent 120 concurrent read requests to `/api/stats` with the same
`X-Forwarded-For` client. All 120 returned `200`. No response returned `429` or
`Retry-After`. The Rust router has no rate-limit layer. This fails the mandatory
allowance on every server-side endpoint.

### F-04 — High — The paid purchase and entitlement boundary do not work

The live **Buy Pro securely** target returned `404` with
`{"error":"enabled factory product","status":404}`. A visitor cannot buy the
advertised $39 license.

The free limits are also enforced only in browser code. Against a disposable
local candidate server, an unauthenticated API caller created six sources and
used `interval_minutes: 15`; all creates returned `201`. The advertised
five-source daily/weekly free tier and paid short schedules therefore do not
hold at the backend boundary.

### F-05 — High — Public claims have no required claim manifest or tests

`.factory/claims.json` is missing. There are no `@claim:<id>` tests and no
declared claim commands to run. The clean test suite passes, but its ten tests
do not provide the required one-to-one claim coverage. Fifteen public claim
groups remain untested under the claims contract; the inventory is below.

### F-06 — High — The container does not use the fleet's durable `/data` mount

The Docker image stores SQLite at `/app/data/change-diff.db` and the README
instructs operators to mount `/app/data`. The product fleet provides durable
storage at `/data`. A fleet redeploy can therefore replace the database even
though a local stop/start test passes.

Related runtime contract gaps are present: the Dockerfile pins
`rust:1.89-alpine`, declares `ARG BUILD_SHA` without the required default, and
explicitly fails an empty build argument. Startup logs do not say which
configuration was generated or supplied. Starting the binary with only `PORT`
does work.

### F-07 — Medium — The landing page does not meet the first-screen or page-order contract

The headline uses a vague “parts that matter” phrase instead of naming the job.
The supporting sentence does not name developers or their monitoring
situation. There is no sample action, no explanation beside the action of what
happens next, and no three separate privacy/offline/price facts.

The landing route also hides the product workflow and paid details behind app
tabs instead of presenting the required page order. Interface headings such as
“Observation desk,” “Choose the signal,” and “Turn quiet watching into a
tighter feedback loop” do not name the section in plain words.
`.factory/copy-audit.md` is missing.

### F-08 — Medium — Route behavior, page titles, metadata, and the 404 page are incomplete

- `/privacy` and `/terms` keep the landing title instead of route-specific titles.
- Browser Back changes `/privacy` to `/` but leaves the Privacy page rendered.
- Route changes do not focus or announce the new `h1`.
- An unknown route correctly returns HTTP `404`, but its body and title are the ordinary landing page. The problem is the broken 404 page, not the expected status.
- `robots.txt` and `sitemap.xml` return `404`.
- Canonical, Open Graph, Twitter card, and apple-touch metadata are absent.
- The footer omits “Built by Param Factory” and the version/build identity.

### F-09 — Medium — Text resize and touch targets miss the accessibility baseline

At 200% text zoom on a 390 px phone viewport, the document had 250 px of
horizontal overflow and primary content extended to 608 px. This causes loss of
a single-column reading flow. Visible targets below 44 px include the 42 px
**Add source** control and the 18 px-high Source footer link. The field error is
announced with `role="alert"`, but the invalid input is not connected to it with
`aria-describedby`.

Automated Axe checks at desktop and phone sizes found zero WCAG A/AA/2.1 AA
violations, and keyboard focus is clearly visible. Those passes do not cover
the resize and target-size failures above.

### F-10 — Medium — Current mobile performance misses the stated budget

A successful fresh mobile Lighthouse run scored 88 for performance, 100 for
accessibility, 100 for best practices, and 100 for SEO. FCP was 1.4 s, LCP 1.7
s, TBT 10 ms, and CLS 0.229. The performance score is below 90 and CLS exceeds
the 0.1 budget. Lighthouse attributed the shift to `.hero-copy` moving after
first paint.

## Public claim inventory

Manual evidence in this review does not replace the required tagged sandbox
test. Each row below lacks exactly one declared `@claim` test, so the untested
claim count is 15.

| # | Public claim group | Current evidence |
| --- | --- | --- |
| 1 | Extract CSS-selected sections, tables, code blocks, and JSON-LD | Partial untagged unit coverage only. |
| 2 | Capture a baseline and create word-level diffs above a threshold | Local baseline worked; no tagged end-to-end test. |
| 3 | Run scheduled checks from 15 minutes to weekly and manual checks | Manual check worked; scheduler behavior is untested. |
| 4 | Respect robots rules and reject private, local, and authenticated targets | Partial untagged unit/source coverage. |
| 5 | Limit responses to 2 MB and extracted content to 250 KB | No observable claim test. |
| 6 | Track unread, reviewed, archived, and useful states | Partial untagged API coverage. |
| 7 | Export the displayed inbox as CSV | Only cell escaping is unit tested; download contents are untested. |
| 8 | Provide a responsive keyboard-accessible interface | Manually checked; no tagged browser test. |
| 9 | Cache an offline shell | Manually passed; no tagged offline browser context test. |
| 10 | Use no analytics, tracking, remote fonts, or runtime CDNs | Manually observed same-origin first load; no privacy claim test. |
| 11 | Limit the free tier to five sources and daily/weekly checks | False at the API boundary. |
| 12 | A $39 license enables unlimited sources and short schedules | Checkout is broken and the API does not enforce the boundary. |
| 13 | Cache license verification for 24 hours without blocking free use | No tagged browser test. |
| 14 | Deleting a source deletes its snapshots and changes | Earlier manual evidence exists; no tagged claim test. |
| 15 | Store state in SQLite and retain it across restart | Local restart passed; no tagged claim test and the fleet mount path is wrong. |

## Checks that passed

### Clean checkout

The detached clean checkout used implementation candidate `c099468`.

| Command | Result |
| --- | --- |
| `npm ci` | Passed; 242 packages installed, zero reported vulnerabilities. |
| `npm run check` | Passed: TypeScript, formatting, and Clippy with warnings denied. |
| `npm test` | Passed: 3 frontend, 4 Rust unit, and 3 Rust API tests. |
| `BUILD_SHA=c099468… npm run build` | Passed; Vite output and optimized Rust binary produced. |

The built JavaScript is 65,755 bytes (25.39 KB gzip) and CSS is 19,189 bytes
(5.22 KB gzip), within the bundle budgets. `frontend/dist/` was produced.

### Normal, invalid, boundary, and recovery paths

- A disposable local source for `https://example.com/` with selector `h1` returned `201`; its first check captured a baseline.
- A repeated check returned the documented 30-second cooldown.
- Invalid name, threshold 101%, and 14-minute interval returned clear `400` errors.
- Threshold 0% and 100%, and intervals 15 and 43,200 minutes, were accepted.
- An invalid CSS selector produced a visible stored error. Editing it to `h1` and checking again captured a baseline and cleared the error.
- A missing change returned a deliberate JSON `404`.
- SQLite retained a created source after graceful stop and restart.
- `/health` returned the candidate build SHA on the local binary and the live build SHA in production.

### Browser, accessibility, privacy, offline, and links

- Fresh desktop and phone contexts had one `h1`, `lang=en`, one `main`, alt text, no console/page errors, and no normal-width overflow.
- Axe found zero violations at both sizes. Tab order starts with the skip link, focus rings are visible, Enter opens the source form, and focus moves to the name input.
- Reduced-motion emulation matched and reduced animation/transition durations to `0.00001s`.
- First load contacted only `change-diff-inbox.sociobot.in`; fonts and art are self-hosted.
- The service worker controlled a reload. Offline reload served the shell and showed a retryable connection error for network-backed data.
- Privacy and Terms return `200`; the GitHub source link returns `200`.
- Security headers include CSP, `nosniff`, frame denial, referrer policy, and permissions policy.
- The factory `verify-url.sh` passed with no console errors.

## Earlier findings

| Earlier finding | Current disposition |
| --- | --- |
| Verification 1: TypeScript check failed | Resolved. `npm run check` passes in the clean candidate checkout. |
| Verification 1: Clippy failed with warnings denied | Resolved. `npm run check` passes. |
| Verification 1: live health could not identify a revision | Resolved for build identity. Live `/health` returns a full SHA; exact live assets match the last implementation commit. |
| Verification 2: zero findings | Superseded. The stricter current contracts and current live checks produce F-01 through F-10. |

## Evidence locations

Review artifacts are outside the repository at
`/work/.evidence/change-diff-inbox-review-1/`, including desktop/phone images,
the Playwright/Axe JSON, factory URL verification, rate-limit response records,
and Lighthouse JSON. The required report copy is
`/work/.evidence/qa-report.md`.

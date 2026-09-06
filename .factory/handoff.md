# Change Diff Inbox — repair 4 handoff

## Result

The repair is live at <https://change-diff-inbox.sociobot.in>.

- Implementation and deployed build: `8e2ac0fa76e88194420a79ae9654064b45974eb3`
- Live revision: `sf-change-diff-inbox--0000013`
- Live image: `sociobotregistry.azurecr.io/sf-change-diff-inbox@sha256:8bdc40055863dd95335b991f8d91fb18d5c5220d18ef9c51c965b66665646163`
- Live `/health`: returns the full implementation SHA above.

The product helps developers review meaningful changes to selected documentation,
status pages, and dashboards. Before scrolling in fresh desktop and phone views,
the job is **Review meaningful page changes**, the audience is developers who
monitor those pages, and the first action is **Try it with sample data**.

## What changed

- Repaired the sole Review 2 finding. The `schedules` claim now creates daily
  and weekly sources, clicks the real **Check now** control, waits for its API
  response, confirms the baseline toast and **Watching** state, and verifies
  the saved source status.
- Added a deterministic HTTP fixture proxy used only by the isolated claim
  runner. The watched URL stays public; the server-side fetch receives known
  HTML and `robots.txt` without an external network dependency. Production
  source validation and fetch behavior are unchanged.
- Updated the schedules sandbox description to document the observed baseline
  outcome rather than merely source controls.
- Wrote public billing metadata to `/work/.evidence/billing-offer.json` and
  copied the verb-first catalog description to `/work/.evidence/catalog-description.txt`.

## Review finding disposition

| Finding | Current disposition |
| --- | --- |
| Review 2 R2-01: schedules test only saw the button | Resolved. The tagged browser claim invokes the button against a deterministic fixture and proves a `baseline` response, visible result, and persisted source state. |
| Review 1 F-01: tenant isolation | Still resolved. The live two-workspace audit sees zero cross-tenant records and cross-delete returns 404. |
| Review 1 F-02: sample sandbox | Still resolved. The live demo has three realistic changes, a persistent label, reset, and no regular-workspace changes. |
| Review 1 F-03: rate limit | Still resolved. A live 50-request burst includes 429 responses with `Retry-After: 1`. |
| Review 1 F-04: paid/free API boundary | Still resolved at the API boundary. Free-limit and signed-Pro entitlement claims pass; sales remain unavailable until external registration. |
| Review 1 F-05 through F-10 and verification 1 findings | Still resolved by the clean claim suite and current live route, accessibility, responsive, metadata, durable-storage, and performance evidence below. |

## Verification

An independent detached worktree at the implementation SHA passed the documented
clean setup:

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

`npm test` passed 3 frontend tests, 4 Rust unit tests, 6 Rust API integration
tests, and 19 browser claim tests. Every exact command in `.factory/claims.json`
was then run independently from that clean checkout; all 19 passed. The repaired
schedules command passed with its browser-driven baseline result. The release
bundle is 26.97 KB gzip JavaScript and 6.21 KB gzip CSS.

The local release binary also passed:

```sh
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" npm run verify:build-identity
```

Current live checks passed:

```sh
/opt/fleet/lib/verify-url.sh https://change-diff-inbox.sociobot.in .factory/evidence/live
node scripts/audit.mjs https://change-diff-inbox.sociobot.in
node scripts/live-audit.mjs https://change-diff-inbox.sociobot.in
node scripts/live-backend-audit.mjs https://change-diff-inbox.sociobot.in
node scripts/live-browser-contract.mjs https://change-diff-inbox.sociobot.in
```

These confirm no load errors, `lang`, title, one `h1`, main landmark, alt text,
zero desktop/mobile Axe violations, keyboard skip-link focus, 200% text resize,
reduced motion, same-origin demo requests, offline shell, legal titles, Back
focus restoration, and a designed deliberate HTTP 404. They also confirm the
first-screen job/audience/action at 1366 px and 390 px, the populated demo/reset
flow, tenant isolation, cleanup, and rate limiting.

The current Lighthouse JSON has performance 98, accessibility 100, best
practices 100, and SEO 100; LCP is 1.74 s, CLS is 0.082, and TBT is 37 ms. The
CLI wrote this complete JSON before Chromium reported a post-run tab crash, so
the command exit was nonzero even though `runWarnings` is empty. This is an
environmental runner limitation, not a hidden successful command.

## Deployment and storage

The durable `sf-change-diff-inbox-data` Azure Files share remains mounted at
`/data`; SQLite and the generated signing key stay there. The deployment
preserved only `PORT=8080`, the mount, and min/max one replica. No secrets were
read or changed. Live health and deployment inspection confirm the immutable
image, durable mount, and one-replica scale are active.

## Known external dependency

The $39 one-time Pro license is still not purchasable because the separate
Sociobot billing-registration operator has not registered the offer. The UI
states that sales are unavailable, no mock checkout exists, and the free core
remains fully usable. The required offer metadata is in
`/work/.evidence/billing-offer.json`; registration is the only remaining
external next step.

# Change Diff Inbox — review 3 handoff

## Result

Strict review of <https://change-diff-inbox.sociobot.in> failed.

- Verdict: **FAIL** — 3 findings and 10 untested public claim groups.
- Severity: 1 high, 2 medium, 0 low.
- Implementation reviewed: `8e2ac0fa76e88194420a79ae9654064b45974eb3`.
- Documentation checkout: `5311eaea1339812d25140697c7d13436c504df9b`.
- Live health identity: `aae3dee9f87bc986c66d5a097442b58f48fa9902`.
- The implementation-to-live and implementation-to-documentation ranges
  contain only reports and stored evidence. Clean frontend assets byte-match
  live.

No product code was changed under this review-only work order.

## What passed

- Fresh desktop and phone first screens name the job **Review meaningful page
  changes**, name the developer audience, and show **Try it with sample data**
  before scrolling.
- The demo contains three realistic changes, keeps its sample-data label,
  resets, and does not change the regular workspace.
- Clean `npm ci`, `npm run check`, `npm test`, and the release build passed.
  All 19 declared claim commands also passed separately.
- Live source and change isolation, invalid-selector recovery, cleanup, health,
  and 429 with `Retry-After` passed. The restart-persistence claim passed.
- Keyboard, focus, 200% resize, reduced motion, offline shell, privacy requests,
  route titles, history focus, links, legal pages, and the deliberate HTTP 404
  behavior passed their reviewed paths.
- Mobile Lighthouse: performance 97, accessibility 100, best practices 100,
  SEO 100; LCP 1.76 s, CLS 0.082, TBT 124 ms.

## Findings to repair

1. **High — claim proof:** six listed claim tests are incomplete and four
   public claim groups have no complete tagged test. See R3-01 for the exact
   inventory. A passing command is not enough; each promised result needs an
   observable assertion in its own claim sandbox.
2. **Medium — heading order:** expanded demo diffs produce `h1 → h3 → h3`.
   Add an h2 grouping level or make Previous/Current h2 headings.
3. **Medium — 404 structure:** keep the intentional 404 status, but add the
   required description, canonical/social/touch metadata and footer build ID.

## Run and verify

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

Then run every exact command in `.factory/claims.json` separately and audit the
assertions against the full public wording. Open `/demo`, expand a change, and
inspect the screen-reader heading list. Request an unknown route and inspect
its head metadata and footer as well as its expected HTTP 404 status.

## Evidence

The review is `.factory/review-3.md`. External evidence is under
`/work/.evidence/change-diff-inbox-review-3/`. The required summary copies are
`/work/.evidence/qa-report.md` and `/work/.evidence/qa-result.json`.

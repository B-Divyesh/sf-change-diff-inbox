# Change Diff Inbox — review 1 handoff

## Result

**FAIL — 10 findings and 15 untested public claim groups.**

The full independent review is in [`review-1.md`](review-1.md). Product code
was not changed. The implementation reviewed is
`c09946838f34cda9aa80f472e548032c5925e43e`; live `/health` reports the later
report-only build SHA `d233c49f8b75cff860856e837981901ad0037080`.
Repository documentation was at
`f2d332bde846a936876ebcd3612dcf7ec718b4b5` when review work began.

## Main gaps

- The public backend has no user or tenant isolation.
- The required one-click sample sandbox is absent.
- API rate limiting does not return `429` with `Retry-After`.
- Checkout returns `404`, and paid limits are not enforced by the backend.
- `.factory/claims.json`, `.factory/demo.md`, and `.factory/copy-audit.md` are missing.
- The container writes SQLite under `/app/data`, not the fleet's `/data` mount.
- Landing copy, routes, metadata, 404 handling, text resize, touch targets, and current CLS miss required contracts.

## Verification performed

From a detached clean checkout at the implementation candidate:

```sh
npm ci
npm run check
npm test
BUILD_SHA=c09946838f34cda9aa80f472e548032c5925e43e npm run build
```

All four commands passed. The local release binary was exercised with only
`PORT`, with a disposable SQLite database across a restart, and through normal,
invalid, boundary, cooldown, and recovery API paths.

The live service was checked in fresh desktop and phone browser contexts with
Playwright and Axe, the factory URL verifier, reduced motion, offline reload,
keyboard navigation, 200% text zoom, route/history behavior, privacy requests,
links, headers, build parity, and a mobile Lighthouse run. The successful
Lighthouse retry scored 88 performance, 100 accessibility, 100 best practices,
and 100 SEO, with CLS 0.229.

## Data safety

No live product data was created or changed. Live stats were zero sources and
zero changes before and after the review. All write-path and restart checks used
a disposable local SQLite database.

## Next steps

Address F-01 through F-10, add one tagged sandbox test for every public claim,
then repeat the full review from a clean checkout and fresh live browser
contexts. A PASS requires zero findings and zero untested claims.

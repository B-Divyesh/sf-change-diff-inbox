# Change Diff Inbox v1 — handoff

## Shipped

- A single-container Rust/Axum + SQLite service serving a Svelte/Vite frontend on `PORT` (8080 by default).
- Public source registry with CSS selector, table, code, and JSON-LD extraction modes; 0–100% noise thresholds; 15-minute through weekly scheduling; manual checks; baseline capture; and source editing/removal.
- Semantic normalization, compact summaries, word-level before/after review, unread/reviewed/archived states, useful/noise feedback, pilot usefulness score, and ungated CSV export.
- Fetch safety: http(s)-only validation, no URL credentials, DNS/private-network rejection, no redirects, `robots.txt` checks, 20-second timeout, 2 MB response and 250 KB extraction limits, a 30-second per-source cooldown, a descriptive user agent, and no script execution or anti-bot behavior.
- First-class empty, loading, backend-error, extraction-error, and offline states. The service worker keeps the application shell available offline; failed fetches never replace the last successful baseline.
- Responsive 390px layout, full keyboard paths, visible focus styling, 44px targets, reduced-motion treatment, semantic landmarks, plain-language `/privacy` and `/terms`, and secure response headers.
- Freemium UI: five daily/weekly sources remain useful for free; a clearly priced $39 one-time Pro license unlocks unlimited sources and 15-minute/hourly schedules. Checkout/verification use only the Sociobot API, verification is cached for one day, returned tokens are removed from the URL, and purchase restoration is available.
- Original generated data-landscape hero with AVIF/WebP responsive exports (12 KB mobile AVIF); provenance and art direction are recorded in `.factory/design.md` and `assets/src/`.
- Multi-stage, non-root Alpine Dockerfile with health check, persistent `/app/data`, migrations on startup, JSON logging, cache headers, compression, panic containment, and graceful shutdown.

## Run and verify

```sh
npm install
npm test
npm run build
npm start
```

The production frontend lands in `frontend/dist/`; the release server lands at `target/release/change-diff-inbox`. Container usage is documented in `README.md`.

Verification completed on 2026-08-27:

- `npm test`: passed — 3 Vitest assertions, 4 Rust watcher unit tests, and 3 API integration tests.
- `npm run build`: passed from the root — Vite production build plus Rust release build.
- Production bundle: 65.76 KB JS / 19.18 KB CSS uncompressed; 25.39 KB / 5.22 KB gzip. Two self-hosted font files total 40 KB. Mobile hero AVIF is 12 KB. Lighthouse total transferred payload was 151 KiB.
- Factory `verify-url.sh` at desktop 1366×900 and mobile 390×844: title present, `lang=en`, exactly one h1, main landmark present, zero missing image alts, zero unlabeled buttons, and zero console/page errors.
- Axe WCAG A/AA/2.1 AA audit at desktop and mobile: zero violations, including zero serious/critical issues. Raw summary: `.factory/evidence/axe.json`.
- Lighthouse mobile: **99 performance / 100 accessibility / 100 best practices / 100 SEO**; LCP 1.9s, TBT 0ms, CLS 0. Raw report: `.factory/evidence/lighthouse.json`.
- Real network smoke: added `https://example.com/` with selector `h1`, then successfully captured a baseline through the API. The disposable source was removed afterward.
- Read-path load smoke: 500 concurrent `/health` requests in 3.703s (~135 requests/second) with no failures.
- `Dockerfile` was inspected and the exact host release build passed. The disposable worker did not provide a Docker daemon, so an image build could not be executed locally.

## Known boundaries

- v1 intentionally fetches server-rendered public HTML only. It does not run page JavaScript, authenticate, render browsers, or bypass bot controls. Client-rendered dashboards need a future explicitly managed browser runner.
- The built-in scheduler is single-instance and SQLite-backed. Horizontal hosted deployments should add a lease/queue before running multiple scheduler replicas.
- Licensing gates convenience and frequency controls in this single-tenant UI. Operators of the MIT self-hosted core retain control of their deployment; the factory must register the billing product before production checkout succeeds.
- Email/webhook delivery and per-user accounts are outside the researched smallest useful product; this v1 is a shared self-hosted inbox.

## Next steps

1. Run a 30-day pilot and use the built-in useful/noise ratings to validate the 80% useful-alert goal.
2. Register `change-diff-inbox` with the Sociobot billing engine and switch the factory hostname to the production API at release (the client selects production automatically on the canonical hostname).
3. If pilot demand supports it, add an isolated managed browser runner with queue leases—without weakening robots, rate-limit, or authentication boundaries.

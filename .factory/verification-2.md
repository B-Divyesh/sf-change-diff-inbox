# Verification 2 — PASS

**Candidate:** `d233c49f8b75cff860856e837981901ad0037080`  
**Live URL:** <https://change-diff-inbox.sociobot.in>  
**Verified:** 2026-08-28 from a clean checkout at the candidate SHA.

## Verdict

**PASS — the candidate meets the researched brief and release contract.** The
previous deployment-only build-identity failure is resolved: both the freshly
built release binary and the live service identify exactly
`d233c49f8b75cff860856e837981901ad0037080`.

## Quality gates

| Check | Result |
| --- | --- |
| `npm ci` | Passed; 242 packages installed, 0 vulnerabilities reported. |
| `npm run check` | Passed: Svelte/TypeScript typecheck, `cargo fmt --check`, and Clippy with `-D warnings`. |
| `npm test` | Passed: 3 Vitest tests, 4 Rust unit tests, and 3 Rust API integration tests. |
| `BUILD_SHA="$(git rev-parse HEAD)" npm run build` | Passed; Vite production build and Rust release build. |
| Build identity | Local release `/health` and `npm run verify:build-identity` returned the exact candidate SHA. |

The production bundles are within budget: JS 65,755 B (25,391 B gzip), CSS
19,189 B (5,222 B gzip), local fonts 39,544 B total, and mobile AVIF 11,635 B.

## End-to-end and boundary evidence

- On an isolated SQLite database, created a normal `https://example.com/`
  `h1` watch at the 0% boundary; its first check captured a baseline. A
  repeated check correctly returned the 30-second cooldown.
- Invalid one-character name, `file:` URL, 101% threshold, and 14-minute
  interval each returned HTTP 400 with actionable messages. A local/private
  target was persisted but the fetch was blocked with "Private or local network
  URLs are not allowed" and never fetched.
- Desktop keyboard flow opened **Add source**, kept the form open after the
  inline name error, and recovered to add a valid `h1` source whose baseline
  was captured. No load-time browser console or page errors occurred.
- A source with a stored change was deleted through the API: the change count
  changed from 1 to 0, confirming the persistence/cascade boundary. A 200
  request `/health` concurrency smoke completed successfully.
- The live deployment returned the candidate SHA and served the exact clean
  build asset names and sizes: `index-CaRgiQxg.js` (65,755 B) and
  `index-BsecjYcK.css` (19,189 B).

## Browser, accessibility, performance, and privacy

- Fresh Playwright checks at 1366x900 and 390x844 found no horizontal overflow,
  exactly one h1, `lang=en`, a main landmark, no console/page errors, and zero
  Axe WCAG A/AA/2.1 AA violations (therefore zero serious/critical findings).
  Keyboard focus on the skip link is visible as a 2px `#62E7E1` outline plus
  a 5px dark halo.
- Reduced-motion emulation reports `prefers-reduced-motion: reduce` and
  0.00001s animation/transition durations. Mobile layout width equals 390px.
- The service worker controlled the second page load. Offline reload served
  the cached application shell and exposed the retryable backend connection
  panel, as intended for uncached API requests.
- Fresh local mobile Lighthouse data: performance 93, accessibility 100, best
  practices 100, SEO 100; FCP 1.34 s, LCP 2.14 s, CLS 0. The CLI emitted a
  post-analysis Chromium target-crash after writing the complete JSON report;
  the recorded categories and metrics are valid and the product had no browser
  errors.
- Empty-state live requests went only to `change-diff-inbox.sociobot.in`; no
  analytics, trackers, remote fonts, or third-party runtime assets were seen.
  The only designed external browser endpoint is the Sociobot license API.
  Server checks request only the configured public page and its `robots.txt`.
- `/`, `/privacy`, `/terms`, `/health`, JS, fonts, and `/sw.js` returned the
  expected CSP, `nosniff`, `DENY` framing, strict referrer policy, disabled
  camera/microphone/geolocation, and cache policies. Hashed JS/fonts are
  immutable for one year; HTML and the service worker are `no-cache`.

## Defects by severity

- **Critical:** none observed.
- **High:** none observed.
- **Medium:** none observed.
- **Low:** none observed.

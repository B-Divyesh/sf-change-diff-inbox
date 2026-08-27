# Verification 1 — FAIL

**Candidate:** `c51139c0ef6eb4b4a4d8e4a3142c9c882b9ca568`  
**Live URL:** <https://change-diff-inbox.sociobot.in>  
**Verified:** 2026-08-27, independent clean detached worktree at the candidate SHA.

## Verdict

**FAIL — do not release this candidate.** The product has a credible working
core, but two available local quality gates fail. The release contract requires
the available type/lint checks to pass.

## Release-blocking defects

### High — frontend type check fails

Command:

```sh
npm exec --workspace frontend tsc -- --noEmit
```

Result:

```text
src/main.ts(7,49): error TS2339: Property 'env' does not exist on type 'ImportMeta'.
```

The production Vite build only transpiles this source, so `npm run build`
passes while the repository's TypeScript configuration reports an error.

### High — Rust lint check fails with warnings denied

Command:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo fmt --check` passed. Clippy failed on `src/watcher.rs` with
`clippy::type_complexity` at line 228 and `clippy::collapsible_if` at line 250.
Both are errors under the requested `-D warnings` quality gate.

### Medium — deployed health endpoint cannot identify the candidate

The live endpoint returned:

```json
{"build":"container","status":"ok"}
```

The Dockerfile sets `BUILD_SHA=container`, rather than an immutable build
commit. This prevents `/health` from providing the required release/build
identity, although the independently rebuilt HTML and JS/CSS asset hashes match
the deployment (below).

## Passed checks and evidence

### Clean checkout, tests, and build

- Created an isolated detached worktree at the candidate SHA and ran `npm ci`:
  244 packages audited, zero reported vulnerabilities.
- `npm test` passed: 3 Vitest tests, 4 Rust unit tests, and 3 Rust API
  integration tests.
- Exact `npm run build` passed: Vite production build followed by
  `cargo build --release`.
- Release output: JS 65,755 B (25,390 B gzip), CSS 19,181 B (5,220 B gzip),
  fonts 39,544 B total, mobile AVIF 11,635 B. These are within the stated
  200 KB JS, 50 KB CSS, 120 KB font, and 300 KB mobile-image budgets.
- `cargo fmt --check` passed. The TypeScript and Clippy failures above mean
  the full type/lint result is failing.

### Functional and backend smoke

- `/health` returned 200 from the freshly built release binary.
- Added and checked `https://example.com/` with selector `h1`: baseline was
  captured successfully.
- Exercised invalid name, `file:` URL, threshold 101%, and 14-minute interval:
  each returned 400 with actionable messages. An invalid CSS selector is
  persisted, then reports `CSS selector is not valid` on check; the UI exposes
  the source error and Edit recovery path.
- Browser UI keyboard smoke: focused **Add source**, activated it with Enter,
  confirmed its inline URL validation keeps the form open, then completed a
  valid source creation/recovery path.
- 200 concurrent `/health` requests against the release binary: 200/200
  successful in 2,110 ms (~95 requests/sec).
- Persistence boundary: inserted a temporary change for a temporary source in
  the disposable local database, deleted that source through `DELETE
  /api/sources/:id`, and confirmed zero orphan changes remained.

### Browser, accessibility, privacy, and PWA

- Playwright desktop (1366×900) and mobile (390×844): exact one `h1`, `main`,
  `lang=en`, correct title, visible cyan two-layer focus (`2px` outline plus
  shadow), no load-time console/page errors, and no third-party requests. The
  only normal first-party requests were app assets, local fonts, API endpoints,
  and responsive hero AVIF.
- Axe at both sizes: 0 violations, therefore 0 serious/critical findings.
- Reduced-motion CSS is present and changes animation/transition duration to
  near-instant; 390px visual inspection showed the navigation and controls
  stack without clipping.
- Runtime/source inspection found self-hosted fonts/assets and no analytics,
  CDN scripts, or tracking requests. The only optional outbound browser API is
  the documented Sociobot license endpoint; server fetches are the configured
  public source and its `robots.txt`.
- Live security responses include CSP, `X-Content-Type-Options: nosniff`,
  `X-Frame-Options: DENY`, referrer policy, permissions policy, immutable
  cache for hashed JS/fonts, and no cache for HTML/service worker. `/privacy`
  and `/terms` returned 200.
- Service worker installs and controls a subsequent page load. An offline
  reload serves the cached application shell, then displays the retryable
  backend-connection state because API responses are intentionally not cached.
  Lighthouse could not be completed in this container: its launcher closed the
  supplied Chromium target (`TargetCloseError`), despite Playwright Chromium
  working for the accessibility and interaction checks.

### Deployment comparison

The live page returned 200 and references:

```text
/assets/index-CmRtfrPF.js
/assets/index-DMn6_7YI.css
```

Those are the exact hashes and byte sizes generated by the clean candidate
build. The deployed HTML (971 B), JS (65,755 B), CSS, security headers,
service worker, empty stats, and visual product shell all matched. The health
build string remains the identity limitation described above.

## Notes / follow-up

1. Fix the TypeScript ambient Vite typing and make the repository's type check
   a declared script/CI gate.
2. Resolve the two Clippy findings (or document a justified lint policy) and
   rerun Clippy with warnings denied.
3. Inject the actual immutable commit SHA at container build time so
   `/health` can attest the deployed build.
4. Re-run this verification, including Lighthouse in an environment with a
   compatible launcher, after the blockers are fixed.

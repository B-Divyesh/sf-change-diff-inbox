# Change Diff Inbox — repair handoff

## Scope

Revalidated the independent verifier's release-blocking report at
`f7b3b6c5e9acd6b2db942941c01cb64dbcd806da` against candidate
`c51139c0ef6eb4b4a4d8e4a3142c9c882b9ca568`, preserving the existing Rust +
Svelte single-container product and researched scope.

## Repairs and regression coverage

- Frontend type checking includes Vite client ambient types and is a declared
  `frontend` `typecheck` command, included in root `npm run check`. This
  catches the verifier's `import.meta.env` error before a Vite build can merely
  transpile it.
- The robots parser has named rule/group types and a single empty-rule guard,
  so the requested denied-warnings Clippy invocation is clean without changing
  robots behavior. `npm run check` runs `cargo fmt --check` and
  `cargo clippy --all-targets --all-features -- -D warnings`.
- Docker requires a validated full 40-character `BUILD_SHA`; `/health` reads
  that compile-time value. The Rust API test asserts the health contract and
  the GitHub workflow builds/runs the actual container then verifies the exact
  `GITHUB_SHA` through `npm run verify:build-identity`.
- The factory container deployment helper was corrected to pass the source
  revision as Docker's `BUILD_SHA` build argument, matching the image contract.
- A fresh mobile Lighthouse run found a real CLS regression (0.229) caused by
  a late self-hosted font swap moving the hero copy. The two preloaded local
  fonts now use `font-display: optional`, preserving first-paint layout while
  retaining local fonts when they are immediately available. The rerun is
  99 performance / 100 accessibility / 100 best practices / 100 SEO, CLS 0.

## Verification

Executed in this clean worker:

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" \
  HEALTH_URL=http://127.0.0.1:18080/health npm run verify:build-identity
/opt/fleet/lib/verify-url.sh http://127.0.0.1:18080 .factory/evidence
node scripts/audit.mjs http://127.0.0.1:18080
```

- `npm ci`: 242 packages installed; 244 audited; zero vulnerabilities.
- `npm run check`: Vite TypeScript, Rust formatting, and denied-warnings
  Clippy passed. `npm test` passed all 3 Vitest plus 4 Rust unit and 3 Rust API
  integration tests. The release build passed.
- The local release `/health` returned the exact built SHA
  `b8a5ea84e248a350325db494ef7455f3ef8608c9` and the identity probe passed.
- Factory browser verifier passed: title, `lang=en`, one h1, main landmark,
  image alt text, labelled buttons, and zero load-time console errors.
- Playwright/Axe at 1366x900 and 390x844 reported zero WCAG A/AA/2.1 AA
  violations. Keyboard Enter opens **Add source** and submits the form; invalid
  input remains open with its actionable inline error. The 390px view has no
  clipping. A service-worker-controlled offline reload serves the shell and
  exposes the retryable backend-connection state.
- No third-party assets, analytics, or runtime CDNs were requested. `/privacy`
  and `/terms` work; response headers include CSP, nosniff, DENY framing,
  strict referrer policy, and disabled camera/microphone/geolocation.
- Current built assets: JS 65,755 B (25,390 B gzip), CSS 19,190 B (5,220 B
  gzip), local fonts 39,544 B, and mobile AVIF 11,635 B — all within budget.
  Lighthouse mobile: 99/100/100/100, CLS 0.

## Run / deploy

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
docker build --build-arg BUILD_SHA="$(git rev-parse HEAD)" -t change-diff-inbox .
docker run --rm -p 8080:8080 -v change-diff-data:/app/data change-diff-inbox
```

The production container deployment and final live identity are recorded after
the deployment step for this handoff commit.

## Known boundaries

The watcher only fetches public server-rendered HTML. It does not execute page
JavaScript, authenticate, bypass robots/access controls, or provide
multi-replica scheduler coordination.

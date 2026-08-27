# Change Diff Inbox — repair handoff

## Release-blocking QA repair

This repair resolves the independent verifier failures recorded at
`f7b3b6c5e9acd6b2db942941c01cb64dbcd806da` for candidate
`c51139c0ef6eb4b4a4d8e4a3142c9c882b9ca568`.

- Added Vite's ambient client types and a declared frontend `typecheck` script;
  `import.meta.env.PROD` now type-checks with `tsc --noEmit`.
- Removed both denied Clippy warnings in the robots parser without changing its
  behavior: named the nested rules/groups types and combined the equivalent
  empty-value condition.
- Made `BUILD_SHA` a required, validated 40-character SHA Docker build
  argument. The compiled `/health` response reports that exact immutable value
  rather than the old `container` placeholder.
- Added a health-route regression plus an exact build-identity probe. CI builds
  the real container with `GITHUB_SHA`, starts it, and asserts `/health` returns
  the same SHA. The factory ACR container-build helper now supplies the current
  full commit SHA as `--build-arg BUILD_SHA=…`.
- Added GitHub Actions quality CI for type checking, formatting, Clippy with
  warnings denied, unit/integration tests, release build, and the container
  identity smoke test.

## Run and verify

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
docker build --build-arg BUILD_SHA="$(git rev-parse HEAD)" -t change-diff-inbox .
docker run --rm -p 8080:8080 -v change-diff-data:/app/data change-diff-inbox
```

`npm run check` includes the Vite TypeScript check, `cargo fmt --check`, and
Clippy with `-D warnings`. To assert an already-running build identity:

```sh
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" HEALTH_URL=http://127.0.0.1:8080/health npm run verify:build-identity
```

## Verification completed

- `npm ci`: passed (244 audited packages; zero vulnerabilities reported).
- `npm run check`: passed (TypeScript, formatting, and denied-warnings Clippy).
- `npm test`: passed (3 Vitest assertions, 4 Rust watcher unit tests, and 3 API
  integration tests).
- `BUILD_SHA=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa npm run build`: passed.
- The resulting release binary returned
  `{"build":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":"ok"}`
  from `/health`; `npm run verify:build-identity` passed against it.
- Factory `verify-url.sh` passed with no load-time console errors: correct
  title/lang, exactly one h1, main landmark, no missing image alts, and no
  unlabeled buttons.
- Playwright Axe desktop (1366×900) and mobile (390×844) audits: zero
  violations at WCAG A/AA/2.1 AA, including zero serious/critical issues.

## Deployment

- ACR build run `ch74` built and pushed
  `sociobotregistry.azurecr.io/sf-change-diff-inbox:16f8ae213325` using
  `BUILD_SHA=16f8ae213325e2d8b5137ee42bd25211d3a5ac54`.
- The fixed container deployment path promoted healthy revision
  `sf-change-diff-inbox--0000003` to 100% traffic.
- Production <https://change-diff-inbox.sociobot.in/health> returned
  `{"build":"16f8ae213325e2d8b5137ee42bd25211d3a5ac54","status":"ok"}`.

## Known limitation

Docker is not installed in this disposable worker, so the local image build and
container run are covered by the new GitHub Actions regression and the
successful factory ACR deployment above. Lighthouse was attempted with the
installed Playwright Chromium, but its launcher crashed the browser tab in this
container; the independent verifier's prior mobile Lighthouse result was
99/100/100/100. The browser, mobile, accessibility, privacy, and offline
behavior were preserved and rechecked through the local release binary.

## Product boundaries

The watcher intentionally fetches public server-rendered HTML only: it does not
execute page JavaScript, log in, solve challenges, or bypass robots and access
controls. The SQLite scheduler remains single-instance; multi-replica hosting
needs a future lease/queue.

# Change Diff Inbox — verification handoff

## PASS

Candidate `d233c49f8b75cff860856e837981901ad0037080` is verified and the live
deployment at <https://change-diff-inbox.sociobot.in> returns that exact SHA
from `/health`.

The independent verification record is in
[`verification-2.md`](verification-2.md). It covers a clean install, all
available checks/tests, production build and identity, representative watcher
and invalid-input flows, persistence and concurrency smoke, desktop/mobile
keyboard and accessibility checks, PWA offline shell, privacy/network policy,
headers/cache policy, bundle budget, Lighthouse, and live parity.

## Re-run

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
PORT=8080 ./target/release/change-diff-inbox
```

For a build-identity check, start the release binary with the `BUILD_SHA`
compiled by the build command, then run:

```sh
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" \
  HEALTH_URL=http://127.0.0.1:8080/health npm run verify:build-identity
```

## Known boundaries

The product intentionally monitors only public, server-rendered HTML. It does
not authenticate to sources, execute page JavaScript, bypass access controls,
or fetch private/local targets. The service-worker shell works offline; API
data remains network-backed and shows a retryable connection state offline.

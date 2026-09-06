# Change Diff Inbox — repair 5 handoff

## Result

Strict review 3 findings are repaired and verified.

- Implementation SHA: `44336e83eba963377f82e8ad34703a68503fadc7`.
- Deployed image: `sociobotregistry.azurecr.io/sf-change-diff-inbox@sha256:4140a273d56d46e5d8829570d32e617955fcbd3f1d3e7aa41d606c7345b29407`.
- Live health: `GET /health` returns build `44336e83eba963377f82e8ad34703a68503fadc7` and status `ok`.
- The documentation handoff is committed separately after this implementation; its SHA is recorded with the final evidence result.

The deployment uses the durable `/data` mount (`sf-change-diff-inbox-data`) and a single replica. It preserves the SQLite database, probes, and runtime configuration.

## Repairs

### Complete public-claim proof

The manifest now contains 22 declared claims, each with one tagged browser test, and every command was run separately from the documented clean setup. The repaired tests assert observable results rather than implementation strings:

- Demo expiry is an exact 24-hour sandbox session, with a matching cookie lifetime.
- A demo workspace cannot read, list, modify, or delete a regular workspace's sources or changes.
- CSS selector, table, code block, and JSON-LD extraction produce the expected meaningful text.
- Both useful and noise review decisions persist and appear in the inbox state.
- CSV output has the exact public header and one record for every displayed source.
- The `/demo` route has its own title and page heading.
- The documented limits allow exactly 40 reads per minute and 10 writes per minute before returning `429` with `Retry-After`.
- A failed check does not replace the prior baseline; the next successful check creates a diff from that preserved text.
- The server fetch path is tested through a controlled proxy: it only reaches the requested source and robots URL. Billing is only reached after an explicit license action.
- Script-bearing content is not executed; authenticated URLs are rejected; a challenge response is not retried or bypassed.

Three claim groups were added for the previously unlisted public safety promises: failed-check baseline preservation, outbound network boundary, and access boundaries.

### Heading and 404 fixes

- An expanded diff now exposes `h1 → h2 → h3 → h3`: the h2 labels the changed-text group, and Previous/Current are h3 headings.
- The static 404 retains its intentional HTTP 404 status and now has a description, canonical URL, Open Graph and Twitter metadata, Apple touch icon, and a Param Factory footer with the injected build ID.
- The build injects the static 404 build ID from `BUILD_SHA`; the container build now includes that script.

## Verification

From a clean dependency install:

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

Results:

- `npm ci`: passed (243 packages, no reported vulnerabilities).
- `npm run check`: passed.
- `npm test`: passed: 3 frontend unit tests, 4 Rust unit tests, 7 Rust API integration tests, and 23 browser tests.
- All 22 commands declared in `.factory/claims.json`: passed when invoked individually.
- Final production build: passed. Initial bundles are 27.02 KB gzip JavaScript and 6.21 KB gzip CSS.
- Live URL verifier: passed (correct title, `lang`, one h1, main landmark, labelled controls, no console errors; 1,068 ms load in the verifier run).
- Axe checks on fresh desktop and 390 px phone `/demo` pages: zero serious or critical violations. The expanded-diff heading order is `H1, H2, H3, H3`.
- Browser contract on live: first screen names the job, audience, and **Try it with sample data** action before scrolling on desktop and phone; 390 px has no horizontal overflow; keyboard skip/focus, reduced motion, Privacy/Terms titles, history focus, offline demo shell, and the designed 404 all passed.
- Live demo: three realistic sample changes appear, the persistent sample-data label remains, reset restores the sample, and regular workspace stats stay at zero.
- Live backend: tenant isolation, cross-workspace mutation denial, cleanup, health, and 429/`Retry-After` passed. Exact allowance check observed 40 reads and 10 writes before rate limiting.
- Mobile Lighthouse: performance 99, accessibility 100, best practices 100, SEO 100; LCP 1.8 s and CLS 0.

External evidence is in `/work/.evidence/change-diff-inbox-repair-5/`. The catalog description copy is `/work/.evidence/catalog-description.txt`.

## Deploy

Build and deploy the committed implementation with the durable one-replica configuration:

```sh
WO_DATA_DIR=/data /opt/fleet/lib/deploy-container.sh change-diff-inbox /work/repo Dockerfile 8080
```

Do not remove `/data` or increase replicas while state is SQLite/process-local.

## Billing status

The free core remains available. The existing Pro offer is still described as a $39 one-time license for unlimited sources and the 15-minute/hour schedules. Sales stay unavailable until the separate billing-registration operator registers the live offer. No checkout or entitlement is claimed as working. Public registration metadata is at `/work/.evidence/billing-offer.json`.

## Earlier findings

All Review 3 findings are resolved by the repairs and tests above. Earlier review findings remain covered: the durable single-tenant SQLite setup, demo separation and reset, explicit local rate limits, route and legal-page metadata, offline shell, privacy boundary, focus and mobile behavior, and the intentional 404 status. The only external dependency is live billing registration; it does not affect the usable free core and is not presented as available.

## Known gap

The product cannot sell the existing Pro offer until the authorised billing-registration operator completes registration. This repair did not add a mock checkout or change paid deliverables.

# Change Diff Inbox — verification 5 handoff

## Result

**FAIL — 1 high finding and zero untested public claims.**

Independent verification reviewed implementation
`44336e83eba963377f82e8ad34703a68503fadc7` and documentation revision
`e6137a75ce96fc729c5752d30d54ee34930dd1c1`. Only the earlier handoff differs
between those revisions. Live `/health` returns the later documentation SHA,
and the live JavaScript and CSS byte-match the clean implementation build.

The live product works across the reviewed desktop, phone, demo, accessibility,
privacy, route, offline, backend, isolation, and rate-limit paths. It is not
accepted because the exact `csv-export` claim command failed nondeterministically
in the clean individual-command run and again during stability testing.

## Finding to repair

`tests/browser/claims.spec.ts` reads the displayed sample source names
immediately after `page.goto('/demo')`. On slow runs, the asynchronous sample
has not rendered, so `displayedSources` is empty even though the export runs
moments later with all three rows. The command failed twice in eight observed
executions and passed six times.

Wait for the three `.change-card` elements before collecting their names. Then
rerun every declared claim command individually from a fresh checkout. Do not
change the live CSV behavior; its downloaded header and three sample rows were
correct.

## Verification summary

- `npm ci`: passed, 243 packages, zero reported vulnerabilities.
- `npm run check`: passed.
- `npm test`: passed with 3 frontend, 4 Rust, 7 API, and 23 browser tests.
- Final candidate build: passed; 27.02 KB gzip JS and 6.21 KB gzip CSS.
- Local health returned the full implementation SHA.
- Twenty-one of 22 individual claim commands passed on their first run.
- The CSV claim command failed on the first run; later and full-suite passes do
  not erase that mandatory-gate failure.
- Fresh live desktop and 390 px phone sessions showed the job, audience, and
  sample action before scrolling.
- Demo population, persistent label, reset, Start for real, and regular-data
  isolation passed. Regular workspace stats remained zero.
- Live Axe found zero violations across home, demo, legal, and 404 routes.
- Keyboard, focus, form error linkage, 200% resize, touch targets, reduced
  motion, offline shell, route titles, legal pages, links, and designed 404
  checks passed.
- Live isolation and cleanup passed. Read limiting allowed 40 then returned
  `429`; write limiting allowed 10 then returned `429`; both supplied
  `Retry-After`.
- Fresh Lighthouse categories were 98 performance and 100 accessibility, best
  practices, and SEO, with LCP 1.70 s and CLS 0.082. The CLI recorded its known
  post-report headless tab crash while Playwright checks remained stable.

Full evidence and disposition of every earlier finding are in
[verification-5.md](verification-5.md). External artifacts are under
`/work/.evidence/change-diff-inbox-verify-5/`.

## Billing status

The free core works. The Pro offer remains unavailable until the separate
billing-registration operator registers it. The live product states this
plainly and does not present checkout as available.

## Product changes

No product code, infrastructure, live source data, billing resource, or secret
was changed during verification. Only this handoff and the verification report
were updated.

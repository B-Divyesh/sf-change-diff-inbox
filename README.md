# Change Diff Inbox

Change Diff Inbox helps developers review meaningful changes to documentation, status pages, dashboards, tables, code blocks, and JSON-LD. It saves selected text instead of screenshots, applies a noise threshold, and puts word changes into a review inbox.

Try the isolated sample at <https://change-diff-inbox.sociobot.in/demo>. It contains three realistic sources and never reads or writes your regular workspace.

## What it does

- Extracts one CSS-selected section, table, code block, or JSON-LD record.
- Saves a baseline and reports word changes at or above the chosen threshold.
- Stores daily or weekly schedules and provides a manual check action.
- Rejects authenticated and private targets and follows matching `robots.txt` rules.
- Limits source responses to 2 MB and extracted text to 250 KB.
- Stores unread, reviewed, archived, useful, and noise decisions.
- Exports every displayed inbox item as CSV.
- Reloads the app shell offline after the first visit. Inbox data still needs the server.

Each browser receives a signed anonymous workspace cookie. Every source, change, and review query is scoped to that workspace. The demo has a different cookie and a temporary database scope that expires after 24 hours.

The free plan accepts five sources with daily or weekly checks. Pro is a $39 one-time license for unlimited sources and 15-minute or hourly checks. The API enforces these limits. Sales are unavailable until the external billing offer is registered; the free product remains usable.

## Stack

Rust 2021 with Axum, Tokio, SQLx, and SQLite serves a Svelte 5 and Vite frontend from one process. The server listens on `PORT`, which defaults to `8080`.

In the production image, SQLite and the generated signing key live under `/data`. A local run uses `./data` when `/data` is absent. `DATABASE_URL`, `DATA_DIR`, `SESSION_SECRET`, `FRONTEND_DIR`, and `BILLING_BASE_URL` can override defaults.

## Run locally

Install Node.js 22 or newer, npm 10 or newer, and current stable Rust. Then run:

```sh
npm ci
npm run build
npm start
```

Open <http://localhost:8080>. The sample is at <http://localhost:8080/demo>.

For frontend development, run `cargo run` and `npm run dev` in separate terminals. Vite proxies `/api` and `/health` to port 8080.

## Test and build

From a clean checkout, run:

```sh
npm ci
npm run check
npm test
BUILD_SHA="$(git rev-parse HEAD)" npm run build
```

`npm test` runs frontend unit tests, Rust unit and API integration tests, and every browser claim in `.factory/claims.json`. Each claim can also run alone with its documented command.

## Container

```sh
docker build --build-arg BUILD_SHA="$(git rev-parse HEAD)" -t change-diff-inbox .
docker run --rm -p 8080:8080 -v change-diff-data:/data change-diff-inbox
```

The image runs as an unprivileged user. `/health` returns the build SHA. The image starts with only `PORT` set and generates its signing key on first boot.

Use one replica with the durable `/data` mount. The API allows 40 read requests per second per forwarded client address and 10 writes per minute. Requests beyond an allowance return `429` with `Retry-After`.

## Safety and privacy

This watcher does not execute page JavaScript, log in, solve challenges, or bypass access controls. A failed check stays visible and does not replace the last good baseline.

The product has no analytics, tracking calls, remote fonts, or runtime CDNs. The server contacts only pages a user adds, their `robots.txt` files, and the Sociobot license service after an explicit license check. See `/privacy` and `/terms`.

## License

MIT. See [LICENSE](LICENSE).

# Change Diff Inbox

Change Diff Inbox is a self-hostable semantic page watcher for developers who monitor documentation, vendor status pages, pricing tables, code examples, and structured data. Instead of sending a screenshot whenever pixels move, it extracts a selected region, normalizes it, filters small changes, and puts meaningful text diffs into a compact review inbox.

## What v1 does

- Watches public HTML pages by CSS selector, table, code block, or JSON-LD.
- Captures a first baseline, then creates word-level semantic diffs only above a configurable noise threshold.
- Runs scheduled checks from 15 minutes to weekly and supports manual checks.
- Respects `robots.txt`, rejects private/local network targets and authenticated URLs, limits responses to 2 MB, and never attempts anti-bot bypasses.
- Tracks unread/reviewed/archived state and whether each alert was useful; exports the inbox to CSV.
- Includes responsive keyboard-accessible UI, offline shell caching, privacy and terms pages, and a Pro license flow through the Sociobot billing API.

The free tier supports five sources and daily/weekly checks. A $39 one-time Pro license unlocks unlimited sources and 15-minute/hourly schedules in the interface. The self-hosted source is MIT licensed; no payment provider is embedded.

## Stack

Rust 2021 with Axum, Tokio, SQLx, and SQLite serves a Svelte 5/Vite frontend from one process. The service listens on `PORT` (default `8080`), stores its SQLite file under `data/` by default, emits structured JSON logs, and shuts down gracefully.

## Run locally

Prerequisites: Node.js 22+, npm 10+, and current stable Rust.

```sh
npm install
npm run build
npm start
```

Open <http://localhost:8080>. For frontend development, run the backend with `cargo run` and, separately, `npm run dev`; Vite proxies `/api` and `/health` to port 8080.

Configuration is environment-only:

| Variable | Default | Purpose |
|---|---|---|
| `PORT` | `8080` | HTTP listen port |
| `DATABASE_URL` | `sqlite://data/change-diff.db?mode=rwc` | SQLite connection URL |
| `FRONTEND_DIR` | `frontend/dist` | Built frontend root |
| `RUST_LOG` | info filters | Structured log filter |

## Test and build

```sh
npm test
npm run build
```

`npm test` runs frontend unit tests plus Rust unit/integration tests. The frontend output is exactly `frontend/dist/`, and the release server is `target/release/change-diff-inbox`.

## Container

```sh
docker build -t change-diff-inbox .
docker run --rm -p 8080:8080 -v change-diff-data:/app/data change-diff-inbox
```

The multi-stage image runs as an unprivileged user and exposes `/health`. Persist `/app/data` in production. Put TLS and access control at your reverse proxy when the deployment should be private.

## Operational boundaries

This is a polite public-page watcher, not a browser automation farm. It does not execute page JavaScript, log in, solve challenges, or bypass robots and access controls. Sites rendered entirely client-side may yield no extractable content. A failing check remains visible on the source record and does not replace the last good baseline.

The scheduler queries at most ten due sources per minute. For a 100-request/second read smoke test, after starting the server you can run `oha -z 10s -c 20 http://127.0.0.1:8080/health`; page-fetch throughput is intentionally governed by source intervals instead.

## Privacy and license

Snapshots and review decisions live only in this deployment's SQLite database. There are no analytics, tracking scripts, remote fonts, or runtime CDNs. Pro checkout and verification use the documented Sociobot API; the browser stores a license token and a daily cached verdict. See `/privacy` and `/terms` in the running app.

## License

MIT. See [LICENSE](LICENSE).

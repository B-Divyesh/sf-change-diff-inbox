# Demo sandbox

## Entry point

Open <https://change-diff-inbox.sociobot.in/demo> or `/demo` on a local server. The landing page links to it with **Try it with sample data**.

## Included sample data

The server creates three sources in a random demo tenant:

- Northstar API limits: a table changes two request limits.
- Acme webhook guide: a code example adds timestamp and algorithm arguments.
- Orbit service status: a JSON-LD status record changes for maintenance.

The resulting inbox contains unread, reviewed, archived, useful, and noise states. The fixture HTML passes through the production extraction, normalization, ratio, and summary functions when the demo is seeded.

## Isolation and reset

Demo requests use `/api/demo/*` and the signed `cdi_demo` cookie. Regular requests use `/api/*` and the separate signed `cdi_workspace` cookie. Every database query includes the selected tenant ID.

Demo tenants expire after 24 hours. **Reset demo** deletes and reseeds only the current demo tenant. **Start for real** returns to the regular workspace; it does not copy sample data.

## Verification

Run `npm run test:claims -- --grep @claim:demo-sandbox`. The test asserts the issued demo session has a 24-hour expiry, changes demo state, resets it, confirms the sample returns, and separately verifies that the regular workspace stays empty.

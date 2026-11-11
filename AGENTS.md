# nibble

A menu bar app (macOS first, Windows and Linux too) that prices a developer's AI usage at API rates and compares it to what their subscription costs each month. `design/HANDOFF.md` holds the product decisions, the mascot, the brand rules and the open questions. Read it before any product or UI work.

## Architecture: hexagonal

Dependencies point inward. The crate boundaries enforce this, so the compiler rejects an import that goes the wrong way.

- `crates/nibble-core`: the domain (plans, usage, pricing, break-even, mood) and the **ports** (traits for anything outside: each Provider's usage and account, the price catalog). It stays pure: no file system, network, environment or wall clock. The current time arrives as a value in `Inputs`. Its own `clippy.toml` bans those APIs. When that lint fires, add a port.
- `crates/nibble-adapters`: implements the ports, for example the Claude Code log reader. It depends on core, never the other way round.
- `crates/nibble-app`: the Tauri v2 shell. It connects the adapters to core, draws the tray icon and serves the popover (React, in `ui/`) a JSON `View`. It contains no business logic: React only words and draws what the `View` says.

Claude Code is the first provider. OpenAI and running several subscriptions at once are planned, so model the provider and the subscription as domain concepts and keep Claude out of core's types. Build core logic test-first.

## Checks

The work is done when `pnpm check` passes. It runs the formatters, clippy, the tests and cargo-deny. Lint settings live in `[workspace.lints]` in `Cargo.toml` and in the `clippy.toml` files. When a lint fires, fix the code. If a lint is genuinely wrong at one spot, put `#[expect(clippy::<name>, reason = "...")]` on the smallest item that covers it.

## Running the app

`pnpm dev` serves the UI on port 5173. Every worktree uses that same port, and a cold Rust build takes minutes. To run the app, keep the chat responsive:

1. Run `lsof -nP -iTCP:5173 -sTCP:LISTEN`. If another nibble worktree's `vite ui` holds the port, kill it and tell the user which worktree it belonged to.
2. Start `pnpm dev` with `run_in_background`, then reply to the user straight away: the build is running and nibble will appear in the menu bar.
3. Read the output when the background job notifies you or the user asks about it.

To work on the popover's look without the Rust build, run `pnpm ui:dev` and open it in a browser. Outside Tauri it shows the sample Views in `ui/src/preview.ts`, picked with `?state=`, e.g. `http://localhost:5173/?state=paidOff`.

## The website

`site/` is nibble's landing page: one static page (`index.html`, `styles.css`, plus a `404.html`), built by Vite. It has two small inline scripts: the light/dark toggle, without which the page follows the system theme, and one that points the download buttons at the file for the visitor's system. `site/vite.config.ts` lists the latest GitHub release's files at build time, so the page needs no network at runtime; without scripts its download buttons get the macOS file. It is independent of the app and the crates. It reuses the popover's fonts and tokens, and `site/vite.config.ts` inlines the drawings from `design/` at build time. `pnpm site:dev` serves it on port 5174 and `pnpm site:build` writes `site/dist`. The page works at a domain root or under a sub-path; `404.html` only at a root. It is live at `nibble.tonytang.dev`, served by Cloudflare Workers as static assets: `wrangler.jsonc` at the repo root configures it, and `.github/workflows/site.yml` deploys it on each push to `main` that touches `site/`, `design/` or the app's icons, and on each published release. Whatever the page says about nibble must be true of the app today; when the app's behaviour or the Price catalog changes, update the page.

## Facts and licensing

- Check plan prices and API rates against the provider's own pricing pages, and record the source URL and the date next to the data.
- nibble is GPL-3.0-or-later, and its owner may sell closed modules later. That only works while the owner holds the copyright to all of the code. So treat Vorssaint as reference reading only and write our own code, and keep dependencies permissive (cargo-deny enforces this).
- For now, usage covers only Claude Code's local logs, and the UI says so.

## Agent skills

### Issue tracker

GitHub Issues on `tonytangdev/nibble`, via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `GLOSSARY.md` and `docs/adr/`. See `docs/agents/domain.md`.

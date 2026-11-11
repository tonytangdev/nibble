# nibble

A menu bar app (macOS first, Windows and Linux too) that prices a developer's AI usage at API rates and compares it to what their subscription costs each month. `design/HANDOFF.md` holds the product decisions, the mascot, the brand rules and the open questions. Read it before any product or UI work.

## The human is a TypeScript developer learning Rust

After any change to Rust code, end your reply with a **From TypeScript** section that explains the change in their terms:

- **Flow**: how data moves through the changed code from start to end, naming the files.
- **Concepts**: map each Rust concept the diff uses to its closest TypeScript equivalent. For example, `trait` is like `interface`, `Result<T, E>` is like a `{ ok: true; value } | { ok: false; error }` union, an `enum` with data is like a discriminated union, a crate is like a package, and `?` is like an early `return` of the error. When the comparison breaks down (ownership, borrowing, lifetimes, `Send`/`Sync`), say so and explain the Rust idea on its own terms.
- **Why**: name the lint, architecture rule or compiler constraint that shaped the code.

The section is done when every Rust concept in the diff that this conversation hasn't covered yet is either mapped to TypeScript or flagged as having no equivalent. The code itself stays idiomatic Rust. The TypeScript framing goes in the reply, not in code comments.

## Architecture: hexagonal

Dependencies point inward. The crate boundaries enforce this, so the compiler rejects an import that goes the wrong way.

- `crates/nibble-core`: the domain (plans, usage, pricing, break-even, mood) and the **ports** (traits for anything outside: each Provider's usage and account, the price catalog). It stays pure: no file system, network, environment or wall clock. The current time arrives as a value in `Inputs`. Its own `clippy.toml` bans those APIs. When that lint fires, add a port.
- `crates/nibble-adapters`: implements the ports, for example the Claude Code log reader. It depends on core, never the other way round.
- The app shell (Tauri v2, not scaffolded yet) connects the adapters to core and renders the UI. It contains no business logic.

Claude Code is the first provider. OpenAI and running several subscriptions at once are planned, so model the provider and the subscription as domain concepts and keep Claude out of core's types. Build core logic test-first.

## Checks

The work is done when `pnpm check` passes. It runs the formatters, clippy, the tests and cargo-deny. Lint settings live in `[workspace.lints]` in `Cargo.toml` and in the `clippy.toml` files. When a lint fires, fix the code. If a lint is genuinely wrong at one spot, put `#[expect(clippy::<name>, reason = "...")]` on the smallest item that covers it.

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

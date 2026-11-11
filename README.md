<div align="center">

<img src="design/nibble-nibbling.svg" alt="nibble, a cream-coloured mouse in round glasses, holding a wedge of cheese" width="160">

# nibble

**Is your Claude subscription paying for itself?**

A menu bar app that prices your Claude Code usage at API rates and compares it to what your plan costs.

[![check](https://github.com/tonytangdev/nibble/actions/workflows/check.yml/badge.svg)](https://github.com/tonytangdev/nibble/actions/workflows/check.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)
![Platform: macOS](https://img.shields.io/badge/platform-macOS%2013%2B-lightgrey.svg)

</div>

nibble, a cheese-loving mouse, eats through a wedge as your usage adds up. It celebrates when your plan has paid for itself this week, and suggests a cheaper or bigger plan when that would suit you better.

## Screenshots

<table>
  <tr>
    <td width="33%"><img src="docs/screenshots/nibbling.png" alt="The popover partway through the week: $15.70 of Claude Code usage against a $23.08 weekly share, 68% paid off"></td>
    <td width="33%"><img src="docs/screenshots/paid-off-dark.png" alt="The popover in dark mode after break-even: $33.12 against $23.08, a 1.4× multiplier and a cheering nibble"></td>
    <td width="33%"><img src="docs/screenshots/celebration.png" alt="The break-even celebration on a teal card: nibble ate the whole wedge"></td>
  </tr>
  <tr>
    <td><b>Nibbling.</b> This week's Claude Code usage at API rates, against your plan's weekly share.</td>
    <td><b>Paid off</b> (dark mode). The plan has covered itself, and the multiplier shows by how much.</td>
    <td><b>Break-even.</b> nibble celebrates once the wedge is gone.</td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/sleepy.png" alt="Sleepy nibble at 13% paid off, suggesting that a smaller plan would have covered the last 30 days"></td>
    <td><img src="docs/screenshots/still-hungry.png" alt="Still-hungry nibble at 6.1×, suggesting a bigger plan for more room"></td>
    <td><img src="docs/screenshots/settings.png" alt="Settings: notify when it pays off, start at login, and what nibble counts"></td>
  </tr>
  <tr>
    <td><b>Sleepy.</b> Underusing the plan, nibble suggests a cheaper one.</td>
    <td><b>Still hungry.</b> Hitting limits, nibble suggests a bigger plan.</td>
    <td><b>Settings.</b> A break-even notification, start at login, and what nibble counts.</td>
  </tr>
</table>

## Features

- **Weekly break-even.** Prices this week's Claude Code usage at API rates and compares it to your plan's weekly share. The week follows Claude's own 7-day limit window, so it refills when your limits do.
- **A cheese wedge in the menu bar.** The icon empties in steps as you use your plan. Once it's paid off, it shows the multiplier, e.g. `1.4×`.
- **Plan detection.** Reads which plan you're on (Pro, Max or Ultra) from your Claude Code sign-in. No setup.
- **Right-sizing suggestions.** If a smaller plan, or paying the API directly, would have covered your last 30 days, nibble says so. If you keep running into your limits, it suggests a bigger plan. "Not now" quiets a suggestion until next week.
- **A break-even notification**, once a week, when your plan pays for itself (can be turned off).
- **Start at login**, light and dark mode.
- **Updates itself, when you say so.** A notification tells you once when a new version is out (click it to open nibble), and a dot on the menu bar icon stays while it is ready; the popover offers to install it and restart.

## How it works

1. nibble reads the session logs Claude Code writes to `~/.claude/projects` (or `$CLAUDE_CONFIG_DIR`) and counts the tokens of every response, by model and token kind.
2. It prices them with a bundled catalog of Anthropic's API rates ([`catalog.json`](crates/nibble-adapters/data/catalog.json)). Each entry records its source page and the date it was checked.
3. It divides your plan's monthly price over the year's weeks (price × 12 ÷ 52) to get a **weekly share**, and compares this week's API value with it.
4. For the suggestions, it looks at the last 30 days of usage, and at the limit usage Claude Code caches in `~/.claude.json`.

Terms like _weekly share_, _paid off_ and _multiplier_ are defined in [GLOSSARY.md](GLOSSARY.md), and the reasoning behind each decision is in [docs/adr](docs/adr).

### What nibble counts

Only **Claude Code**, from its logs on your computer. Chats on claude.ai and in the desktop app leave no local record, so they are not included. If you use those too, your real usage is higher than nibble shows. Models missing from the price catalog are counted but not valued, and the amount is then marked as a lower bound.

Team and Enterprise seats see their API value, without break-even or suggestions, because their pricing works differently.

## Privacy

nibble runs on your computer, and nothing about your usage leaves it. Its only network request is the update check: at launch and once a day it fetches `latest.json` from this repository's GitHub Releases. Turn that off in Settings with "Check for updates". From Claude Code's logs it keeps only token counts, model names and timestamps, never your prompts or replies. Its own history (hourly totals for about 13 weeks) lives in the app's data folder and can be deleted at any time; nibble rebuilds what it can from the logs.

## Installation

There are no prebuilt releases yet. To build nibble from source you need:

- [Rust](https://rustup.rs) (rustup installs the pinned toolchain automatically)
- [Node.js](https://nodejs.org) and [pnpm](https://pnpm.io)
- macOS 13 or later. Windows and Linux builds compile in CI but are not supported yet; on Linux, install [Tauri's system dependencies](https://v2.tauri.app/start/prerequisites/) first.

```sh
git clone https://github.com/tonytangdev/nibble.git
cd nibble
pnpm install
pnpm build    # the app bundle lands in target/release/bundle
```

You also need Claude Code installed and signed in, or nibble has nothing to read.

## Roadmap

- Windows and Linux
- OpenAI and other providers
- Several subscriptions at once
- Prebuilt releases

## Development

```sh
pnpm dev      # build and run the app
pnpm ui:dev   # just the popover, in a browser, with sample data
pnpm check      # everything: formatters, lints, tests, licence checks and the site build
pnpm app:check  # just the app's checks
pnpm site:check # just the landing page's checks, ending with its build
pnpm site:dev   # the landing page in site/, on port 5174
```

`pnpm ui:dev` serves the popover at <http://localhost:5173> with no Rust build. Pick a sample state with `?state=`, e.g. `?state=paidOff`; the samples are in [`ui/src/preview.ts`](ui/src/preview.ts). Set `NIBBLE_DATA_DIR` to keep settings and history in another folder, e.g. for demo data.

The code is laid out hexagonally, so the domain never depends on the file system, the UI or Claude:

```text
crates/
├── nibble-core/       # the domain: plans, usage, pricing, break-even, mood, and the ports
├── nibble-adapters/   # implements the ports: Claude Code's logs and account, the price catalog
└── nibble-app/        # the Tauri v2 shell: tray icon, popover window, wiring
ui/                    # the popover, in React; it only draws the View the shell sends
```

Releases are built, signed and notarized by CI from a version tag; see [docs/releasing.md](docs/releasing.md).

## Contributing

Issues and pull requests are welcome on [GitHub](https://github.com/tonytangdev/nibble/issues). Please run `pnpm check` before opening a pull request.

The owner keeps the copyright to all of nibble's code (see [ADR-0005](docs/adr/0005-gpl-and-full-ownership.md)), so contributors are asked to sign a CLA. For the same reason, please don't copy code from other projects, and keep new dependencies permissively licensed; `cargo deny` checks this.

## Support

If nibble is useful to you, you can [buy me a coffee](https://buymeacoffee.com/tonytangdev) ☕

You can find my other apps at [tonytang.dev/apps](https://tonytang.dev/apps/).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

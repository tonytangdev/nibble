# Handoff: nibble, a macOS menu bar app

The next session runs inside the new **nibble** project, which the user is creating now. Its job is to start building the app.

## What nibble is

nibble is an always-on macOS menu bar app for developers. It answers one question: "Is my Claude subscription paying for itself?" It does that by pricing your usage at API rates and comparing the total to your plan's monthly price. When you are underusing the plan, it suggests a cheaper one. When you keep hitting limits, it suggests a bigger one.

The mascot is **nibble**, a cream-colored mouse wearing the round glasses from the tonytang.dev logo, with yellow pupils. Each plan is a wedge of cheese that the mouse eats through. The app and the mascot share the name.

## Decisions already made by the user

- **Name:** `nibble`. The brand is tonytang.dev, and the likely home is `nibble.tonytang.dev`. Domain and trademark availability have **not** been checked.
- **Tone:** cute and cozy. **Platform:** macOS menu bar, always on. **Audience:** mainly developers. **Language:** English only.
- **Mascot reacts both ways:** it celebrates at break-even, and it is sleepy or sad when suggesting a downgrade.
- **The user is not contributing to Vorssaint.** nibble is a standalone app.
- **Paid-off menu bar icon is "option D":** an empty wedge outline with a ✓ inside, plus a multiplier as text, e.g. `1.2×`. The user rejected the earlier version, which showed only nibble's head, as not self-explanatory.
- **The user approved the rest of the mascot board.**

## Artifacts (copy these into the new project, because the scratch folder may get cleaned)

Everything is in `/Users/tonytang/.t3/scratch/2026-11-11-help-me-find-a-new-03f6073f/`:
- `nibble/nibble-moods.html` is the approved board: 4 moods plus the menu bar strip.
- `nibble/nibble-{sleepy,nibbling,paid-off,still-hungry}.svg` are the mascot illustrations.
  - `sleepy`: underused, suggest a downgrade.
  - `nibbling`: progress toward break-even.
  - `paid-off`: the break-even pop-up, on a teal card.
  - `still-hungry`: hitting rate limits, suggest an upgrade.
- `nibble/menubar/{wedge-100,wedge-66,wedge-33,crumbs,paid-off}.svg` are monochrome template icons with a 20×20 viewBox, shown at 18px.
- `scripts/gen.py` generates all of the above: `python3 -I scripts/gen.py nibble`. Palette constants and the `wedge(level)` mask logic are in there, and the wedge cut can be computed for any percentage.
- `nibble/paid-off-options.html` and `scripts/paidoff.py` are the rejected icon alternatives, kept for reference only.

## Brand source of truth

The user's site repo is `/Users/tonytang/Documents/github/tonytangdev/tonytangdev-v2`. Read `DESIGN.md` for the palette, fonts and the rule that yellow is used only as a signal, and `PRODUCT.md` for voice. The logo mark is in `public/apps/secret.svg` and `public/favicon.svg`.
- Teal `oklch(0.42 0.085 197)`, Teal Deep `oklch(0.34 0.08 202)`, Cream `oklch(0.96 0.025 95)`, Signal Yellow `oklch(0.86 0.16 92)`, Ink-Fixed `oklch(0.17 0.022 200)`.
- Fonts: Bricolage Grotesque for display, Public Sans for body, JetBrains Mono for labels.
- In nibble, yellow is used only for the cheese and the pupils. The full teal background is used only for the break-even moment.

## Technical findings

- **Vorssaint** ([github.com/vorssaint/vorssaint-utils](https://github.com/vorssaint/vorssaint-utils), Swift, **GPL-3.0**) already has a Claude usage tracker that is a useful reference. **Read it for ideas only.** Copying its code would force nibble to be GPL-3.0.
  - `Sources/Vorssaint/Services/AgentUsage/Harness/ClaudeReader.swift` reads Claude Code's local logs.
  - `Services/AgentUsage/ClaudeAccountProfile.swift` reads the plan tier from the Claude config: `organizationType` plus `organizationRateLimitTier` / `userRateLimitTier`, e.g. `max_20x`. It looks in `~/.claude*` folders.
  - `Resources/agent-prices.json` holds per-model API prices plus a monthly price for each plan.
- **Usage scope is still open.** The suggestion is to measure **Claude Code usage only**, since its logs are on disk, and to say so in the UI. claude.ai and desktop chats leave no local token logs. The user has not confirmed this.
- Verify current Claude plan prices and API rates from primary sources. Do not rely on memory.

## Open questions to settle early

1. Usage scope: Claude Code only, or also estimate claude.ai usage?
2. Should the menu bar show text before break-even (e.g. `62%`), or only the icon? The suggestion is icon only until break-even, then the `1.2×` multiplier.
3. Should the wedge shrink smoothly (computed live) or in fixed steps (100/66/33/crumbs)?
4. What should trigger the "still hungry" (upgrade) state? Rate-limit hits per week, or something else?
5. Tech stack. The default assumption is SwiftUI with `MenuBarExtra`, macOS 14+, Apple Silicon. Confirm this with the user.

## Suggested skills

- `grilling`: work through the open questions above before writing code.
- `domain-modeling`: set up GLOSSARY.md covering plan, usage, API value, break-even, multiplier and mood states, and record ADRs such as "Claude Code logs only."
- `research`: confirm the Claude Code log format and location, plan-tier detection, and current plan prices and API rates.
- `codebase-design`: plan the module boundaries: log reader, pricing, plan detection, the logic that picks a mood, and the UI.
- `tdd`: build the pricing and break-even logic test-first.
- `writing-for-agents`: write the new project's CLAUDE.md / AGENTS.md.
- `prototype`: only if the popover or the break-even animation needs a throwaway UI exploration.

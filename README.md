# nibble

A menu bar app that prices your AI usage at API rates and tells you whether your subscription is paying for itself. nibble, a cheese-loving mouse, eats through a wedge as your usage adds up, celebrates when the plan has paid for itself, and suggests a cheaper or bigger plan when that would suit you better.

For now nibble measures Claude Code only, from its local logs. macOS comes first; Windows and Linux are planned.

## Screenshots

<table>
  <tr>
    <td width="33%"><img src="docs/screenshots/nibbling.png" alt="The popover partway through the week: $15.70 of Claude Code usage against a $23.08 weekly share of Max 5x, 68% paid off"></td>
    <td width="33%"><img src="docs/screenshots/paid-off-dark.png" alt="The popover in dark mode after break-even: $33.12 against $23.08, a 1.4× multiplier and a cheering nibble"></td>
    <td width="33%"><img src="docs/screenshots/celebration.png" alt="The break-even celebration on a teal card: nibble ate the whole wedge"></td>
  </tr>
  <tr>
    <td><b>Nibbling.</b> This week's Claude Code usage at API rates, against your plan's weekly share.</td>
    <td><b>Paid off</b> (dark mode). The plan has covered itself, and the multiplier shows by how much.</td>
    <td><b>Break-even.</b> nibble celebrates once the wedge is gone.</td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/sleepy.png" alt="Sleepy nibble at 13% paid off, suggesting that Pro would have covered the last 30 days"></td>
    <td><img src="docs/screenshots/still-hungry.png" alt="Still-hungry nibble at 6.1×, suggesting Max 20x for more room"></td>
    <td><img src="docs/screenshots/settings.png" alt="Settings: notify when it pays off, start at login, and what nibble counts"></td>
  </tr>
  <tr>
    <td><b>Sleepy.</b> Underusing the plan, nibble suggests a cheaper one.</td>
    <td><b>Still hungry.</b> Hitting limits, nibble suggests a bigger plan.</td>
    <td><b>Settings.</b> A break-even notification, start at login, and what nibble counts.</td>
  </tr>
</table>

## Development

```sh
pnpm install
pnpm dev      # run the app
pnpm check    # formatters, lints, tests and licence checks
```

## Support

If nibble is useful to you, you can [buy me a coffee](https://buymeacoffee.com/tonytangdev) ☕

You can find my other apps at [tonytang.dev/apps](https://tonytang.dev/apps/).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

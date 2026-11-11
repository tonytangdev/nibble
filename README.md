# nibble

A menu bar app that prices your AI usage at API rates and tells you whether your subscription is paying for itself. nibble, a cheese-loving mouse, eats through a wedge as your usage adds up, celebrates when the plan has paid for itself, and suggests a cheaper or bigger plan when that would suit you better.

For now nibble measures Claude Code only, from its local logs. macOS comes first; Windows and Linux are planned.

## Development

```sh
pnpm install
pnpm dev      # run the app
pnpm check    # formatters, lints, tests and licence checks
```

Releases are built and signed by CI from a version tag; see [docs/releasing.md](docs/releasing.md).

## Privacy

nibble reads Claude Code's logs on your computer and sends nothing about them anywhere. Its only network request is the update check: once a day it fetches `latest.json` from this repository's GitHub Releases. Turn that off in Settings with "Check for updates".

## Support

If nibble is useful to you, you can [buy me a coffee](https://buymeacoffee.com/tonytangdev) ☕

You can find my other apps at [tonytang.dev/apps](https://tonytang.dev/apps/).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

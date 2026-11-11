# The update check is nibble's only network call, and it is opt-out

nibble ships through GitHub Releases, signed and notarized, and updates itself with Tauri's updater. To find a new version it fetches `latest.json` from the repository's latest release, at launch and once a day; release builds only. That request carries nothing about the user's usage, and it is the only one nibble makes: the logs, the history and the settings never leave the computer. The check is on by default, because a version without working updates strands its users, and a Settings switch turns it off. Installing always waits for the user's "Install & restart", since a menu bar app that restarts by itself looks broken.

Anything else that would reach the network (telemetry, crash reports, fetching prices) needs its own decision.

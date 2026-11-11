# Releasing nibble

A release is a signed, notarized macOS app for Apple Silicon (macOS 13+) on a
GitHub Release, plus the `latest.json` the in-app updater reads. Intel Macs are not supported. Windows and Linux come later.

## Once: signing secrets

Needs an active Apple Developer Program membership (individual).

```sh
scripts/release-secrets.sh
```

The wizard creates the Developer ID Application certificate, the App Store Connect API key for
notarization and the updater signing key. It saves them as GitHub secrets, writes the updater's
public key into `crates/nibble-app/tauri.conf.json`, and asks you to back everything up in your
password manager. Commit that public key and merge it to main.

Keep the updater key forever. Installed copies accept only updates signed with it, so losing it
or replacing it leaves every existing user on their current version.

The Developer ID certificate lasts five years. To renew it, delete `~/.nibble-release/developer-id.*`
and run the wizard again.

## Every release

1. Bump `version` in the root `Cargo.toml` (the app takes its version from there), merge to main.
2. Tag the merge commit on main and push the tag:
   ```sh
   git tag v0.1.0 && git push origin v0.1.0
   ```
3. The `release` workflow checks that the tag is on main, matches the version and that every
   secret is set, runs the app's checks (`app-checks.yml`, on macOS), then builds, signs, notarizes and uploads a **draft**
   release with generated notes.
4. Download the `.dmg` from the draft, install it, and open it: macOS should open it without a
   warning. Edit the notes, then publish. Publishing is what makes it `latest`, for both the
   download links and the updater. Publishing also redeploys the website, whose download list
   shows the latest release's files.

## Trying a build without the secrets

Actions → **build** → Run workflow makes an unsigned Apple Silicon `.dmg` as a workflow artifact,
never a release. macOS blocks it as downloaded; open it with `xattr -cr /Applications/nibble.app`.

Locally, `pnpm build` makes an unsigned build for your own Mac in `target/release/bundle/`.

## Updates

nibble checks `https://github.com/tonytangdev/nibble/releases/latest/download/latest.json` when
it starts and once a day, unless "Check for updates" is off in Settings. Development builds never
check. When a newer version exists, the popover offers to install it and restart.

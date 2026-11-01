# Nyashk Launcher v1.1.0

One app to install and update all Nyashk plugins (NyashkSynth, NyashkOpto, NyashkTune).

- Plugin list by category with installed and latest version, and an "update available" badge
- Check for updates, update one plugin, update the selected ones or all at once
- Install any older or newer version from a dropdown (downgrade is the same menu)
- Release notes ("What's new since your version") come straight from GitHub Releases
- Downloads from GitHub, verified with the SHA-256 checksum; the old version is only replaced once the new one is fully unpacked
- Settings: plugin folder (with Browse), startup check, pre-releases, optional GitHub token, cache
- Small single-file app for Linux, Windows and macOS
- Checks for a new launcher version too: one click downloads it, replaces itself and restarts
- Re-checks plugins in the background while open (interval in Settings) and shows a desktop notification; the window title shows the number of pending updates
- Plugin preview pictures on the cards and a page for every plugin: gallery, description, features and full release history

## What's new in 1.1.0
- Five new plugins: NyashkChannel, NyashkEQ, NyashkAir, NyashkPre, NyashkMeter.
- New "Analysis" category.
- The plugin list now comes from `plugins.json` on GitHub: new plugins no longer need a launcher update.
- Rounded app icon.

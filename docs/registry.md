# The plugin registry

> **Not built yet.** The recorder's plugin store and the registry repository
> are the next step. This page describes how they're designed to work, so
> plugins released now will fit. Until then, install a plugin by hand: unpack
> its release into the plugins folder (below), where the recorder's Plugins
> page finds it, or add its executable with **Add from a file** on that page.

The registry is a GitHub repository, `TrunkRecorder/plugins`, listing the
plugins the recorder's plugin store offers. Each entry pins an exact release
of a plugin by checksum. The recorder installs only files that match, so what
users get is what was reviewed.

## An entry

Each plugin has a file, `plugins/<id>.json`:

```json
{
  "id": "openmhz",
  "name": "OpenMHz",
  "description": "Uploads calls to OpenMHz.",
  "repository": "https://github.com/TrunkRecorder/trunk-plugin-openmhz",
  "tier": "official",
  "version": "1.0.0",
  "api": 1,
  "assets": {
    "x86_64-unknown-linux-gnu": {
      "url": "https://github.com/TrunkRecorder/trunk-plugin-openmhz/releases/download/v1.0.0/openmhz-1.0.0-x86_64-unknown-linux-gnu.tar.gz",
      "sha256": "9f2c…"
    },
    "aarch64-unknown-linux-gnu": { "url": "…", "sha256": "…" },
    "universal-apple-darwin": { "url": "…", "sha256": "…" },
    "x86_64-pc-windows-msvc": { "url": "…", "sha256": "…" }
  }
}
```

`tier` is `official` for plugins maintained with the recorder, and `community`
for everyone else's. The store shows the difference.

You don't write the `assets` by hand. A script in the registry fills them in
from your release's `SHA256SUMS`.

## Getting listed

1. Release your plugin with the template's release workflow (see
   [Releasing](releasing.md)).
2. Open a pull request on the registry that adds `plugins/<id>.json`. Run the
   registry's `add-release` script to fill in the assets:
   `./add-release https://github.com/you/trunk-plugin-pager v0.1.0`.
3. A maintainer reviews it.

What review looks for:

- **The source is public**, and the release was built from it by GitHub
  Actions, not uploaded from someone's computer.
- **It does what it says**, and nothing else. In particular, it sends no data
  anywhere the user didn't configure.
- **Secrets are marked** `x-secret`, and never logged.
- **It copes with missing M4A** if it asks for M4A (see [Audio](audio.md)).
- **It starts with its default settings**, or explains in `start` what to set.
- **It's licensed**, and the id isn't taken.

## Updating

Release the new version, then open a pull request that updates your entry's
`version` and `assets`. `add-release` does it. A registry workflow also
watches listed plugins for new releases and opens these pull requests on its
own.

The store shows **Update available** when the registry has a newer version
than the one installed. Users update when they choose. Settings carry over.

## How the recorder installs a plugin

1. It downloads the archive for its platform from the entry's `url`.
2. It checks the archive's SHA-256 against the entry, and stops if they differ.
3. It unpacks the archive into `plugins/<id>/` in its config folder (next to
   `config.json`).
4. It runs `<id> --describe`, and checks that the id matches the entry and that
   it supports the plugin's `api`.
5. It shows the plugin's settings form, and the user turns it on.

The recorder ships with a copy of the registry, so the store works offline.
It fetches the latest registry when the store is opened.

## Unlisted plugins

A plugin doesn't have to be in the registry to run. The store can install one
from a GitHub release URL, with a warning that nobody has reviewed it. And for
development, **Add from a file** on the Plugins page runs any executable. See
[Getting started](getting-started.md#6-run-it-inside-your-recorder).

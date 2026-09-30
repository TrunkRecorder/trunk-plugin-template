# Getting started

This walks you from the template to your own plugin running inside your
recorder. It takes about fifteen minutes, most of it compiling.

## What you need

- **Rust**, from [rustup.rs](https://rustup.rs). Any recent stable version.
- **Trunk Recorder Lite**, installed and set up to record something. You'll
  test against calls it has already recorded, so you don't need a radio
  attached while you work.
- **Optional: ffmpeg**, if your plugin wants M4A audio (see [Audio](audio.md)).

## 1. Make your repository

On GitHub, press **Use this template** on this repository. Name the new
repository after your plugin, for example `trunk-plugin-pager`. Clone it.

## 2. Name your plugin

A plugin has an **id**: lowercase letters, digits and dashes, like `pager` or
`mqtt-status`. The id names the executable, the release files, and the
plugin's entry in the registry. Once people have installed your plugin, the id
can't change. Pick it now.

Change these:

| File | Field | Example |
|---|---|---|
| `Cargo.toml` | `name`: the id | `pager` |
| `Cargo.toml` | `description`: one sentence, shown in the plugin store | `Pages your phone when a talkgroup goes active.` |
| `Cargo.toml` | `repository` | `https://github.com/you/trunk-plugin-pager` |
| `Cargo.toml` | `authors`, `license` | |
| `src/main.rs` | `name` in `manifest()`: the display name | `Pager` |
| `LICENSE` | Your license | |

Then tidy up what belongs to the template, not your plugin:

- **Delete `docs/`**. It documents the template; it lives on in the template's
  repository, where it's kept up to date.
- **Rewrite `README.md`** for your plugin's users. See
  [below](#a-readme-for-your-plugin).
- **Delete `examples/settings.json`**, or rewrite it with your settings.

## 3. Build it and ask it who it is

```sh
cargo build
./target/debug/pager --describe
```

`--describe` prints the plugin's **manifest**: its id, version, the events it
subscribes to, and the schema of its settings. The recorder reads this to
decide how to run the plugin and how to draw its settings form. See
[How plugins work](how-plugins-work.md).

## 4. Run it against calls you've recorded

`trunk-lite plugin run` starts a plugin the way the recorder does and sends
it calls from disk, each as a `call.concluded` event. It prints everything the
plugin says back:

```sh
trunk-lite plugin run ./target/debug/pager ~/TrunkRecorderLite --limit 5
```

```
M4A: none
[pager] status ok: running
[pager] ✓ sys1/2026/9/30/101-1790771550_857587500
[pager] ✓ sys1/2026/9/30/1039-1790771535_858587500
…
```

To give it settings, put them in a file, like [`examples/settings.json`](../examples/settings.json):

```json
{
  "config": { "file": "calls.jsonl" },
  "systems": { "sys1": { "label": "County" } }
}
```

```sh
trunk-lite plugin run ./target/debug/pager ~/TrunkRecorderLite --settings settings.json
```

`config` holds the plugin's settings. `systems` holds its settings for each
system, keyed by the system's short name. See [Settings](settings.md).

## 5. Write your plugin

Start with `src/main.rs`. The example shows each part a plugin has:

- `Config` and `SystemConfig`: the settings, with doc comments that become the
  form's labels.
- `manifest()`: the plugin's name and the events it subscribes to.
- `start()`: checks the settings and sets up. Returning an `Err` shows the user
  what's wrong.
- One method per event you subscribed to, here `call_concluded()`.
- `shutdown()`: finishes up before the recorder stops.

Read [How plugins work](how-plugins-work.md) before you do anything slow, such
as network calls, in an event method.

Keep `cargo test` passing as you go. The example's tests show how to drive a
plugin without the recorder. See [Testing](testing.md).

## 6. Run it inside your recorder

Tell the recorder about your build in `plugins.json`. The file sits next to the
recorder's `config.json`:

- **macOS**: `~/Library/Application Support/trunk-lite/`
- **Linux**: `~/.config/trunk-lite/`
- **Windows**: `%APPDATA%\trunk-lite\`

```json
{
  "plugins": {
    "pager": {
      "enabled": true,
      "path": "/Users/you/code/trunk-plugin-pager/target/release/pager",
      "config": { "file": "calls.jsonl" },
      "systems": { "sys1": { "label": "County" } }
    }
  }
}
```

`path` points the recorder at your build instead of an installed copy.
`trunk-lite plugin list` shows what the recorder makes of the file.

Start recording. Your plugin's log lines appear in the recorder's log,
prefixed with its id. After you rebuild, stop and start recording to pick up
the new binary.

## A README for your plugin

Users read it on GitHub before they install, and reviewers read it before
listing your plugin. Cover:

```markdown
# Pager

Pages your phone when a talkgroup goes active. A plugin for Trunk Recorder Lite.

## What it needs
An ntfy.sh topic (free). Nothing to install.

## Settings
| Setting | |
|---|---|
| Topic | Your ntfy.sh topic. |
| Talkgroups | The talkgroups to page for, by number. |

## What it sends where
A notification with the talkgroup's name to ntfy.sh, for each call on your talkgroups. Nothing else.
```

"What it sends where" matters: users are trusting your plugin with their
recorder, and the [registry](registry.md) review checks it.

## 7. Release it

Tag a version and push the tag:

```sh
git tag v0.1.0
git push --tags
```

The release workflow builds your plugin for Linux (x86-64 and ARM64), macOS and
Windows, and attaches the builds to a GitHub release along with checksums. See
[Releasing](releasing.md). Then [submit it to the registry](registry.md), so
people can install it from the recorder's plugin store.

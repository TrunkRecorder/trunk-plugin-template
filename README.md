# Trunk Recorder Lite plugin template

A starting point for a [Trunk Recorder Lite](https://github.com/TrunkRecorder/trunk-recorder-lite)
plugin in Rust: a working example plugin, tests, CI, and a release workflow
that builds for every platform the recorder runs on.

A plugin is a small program of its own. The recorder starts it, tells it what
happens (a call starts, a recorded call lands on disk, a radio registers, …),
and stops it. Plugins watch; they don't change what gets recorded. Uploaders
(OpenMHz, Broadcastify), streamers, loggers and notifiers are all plugins.

The example here, **call-log**, writes a line of JSON for every recorded call.

## Quick start

1. **Use this template** on GitHub (or copy it) to make your plugin's repository.
2. In `Cargo.toml`, set `name` (your plugin's id), `description`, `repository`
   and `authors`. In `src/main.rs`, set the display `name` in `manifest()`.
3. Build and look at what the recorder will see:
   ```sh
   cargo build
   ./target/debug/call-log --describe
   ```
4. Run it against calls you've already recorded:
   ```sh
   trunk-lite plugin run ./target/debug/call-log ~/TrunkRecorderLite --limit 5
   ```
5. Replace the example with your plugin, and keep the tests passing:
   `cargo test`.
6. Tag a release (`git tag v0.1.0 && git push --tags`); the workflow builds it
   for Linux, macOS and Windows. Then [submit it to the registry](docs/registry.md).

## Documentation

| | |
|---|---|
| [Getting started](docs/getting-started.md) | From this template to a plugin running in your recorder |
| [How plugins work](docs/how-plugins-work.md) | The process, its lifecycle, threads, failures |
| [Settings](docs/settings.md) | Config structs and the settings form drawn from them |
| [Events](docs/events.md) | Everything a plugin can subscribe to, field by field |
| [Audio](docs/audio.md) | WAV and M4A, and what to do when M4A isn't there |
| [Testing](docs/testing.md) | Unit tests, and running against real calls |
| [Releasing](docs/releasing.md) | Versions, the release workflow, platforms |
| [The plugin registry](docs/registry.md) | How plugins get listed, installed and updated |
| [Protocol](docs/protocol.md) | The wire format, for plugins in other languages |

The SDK's API reference is on [docs.rs/trunk-recorder-plugin](https://docs.rs/trunk-recorder-plugin).

## License

The template is MIT-licensed; your plugin can use any license you like.

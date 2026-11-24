# Trunk Recorder Pro plugin template

A starting point for a [Trunk Recorder Pro](https://github.com/TrunkRecorder/trunk-recorder-pro)
plugin in Rust: a working example plugin, tests, CI, and a release workflow
that builds for every platform the recorder runs on.

A plugin is a small program of its own. The recorder starts it, tells it what
happens (a call starts, a recorded call lands on disk, a radio registers, …),
and stops it. Plugins watch; they don't change what gets recorded. Uploaders
(OpenMHz, Broadcastify), streamers, loggers and notifiers are all plugins.

The example here, **call-log**, writes a line of JSON for every recorded call.
For complete plugins built from this template, see the
[OpenMHz](https://github.com/TrunkRecorder/trunk-plugin-openmhz),
[Broadcastify](https://github.com/TrunkRecorder/trunk-plugin-broadcastify) and
[Rdio Scanner](https://github.com/TrunkRecorder/trunk-plugin-rdioscanner)
uploaders, [simplestream](https://github.com/TrunkRecorder/trunk-plugin-simplestream)
(live audio) and [upload-script](https://github.com/TrunkRecorder/trunk-plugin-upload-script).

## Quick start

1. **Use this template** on GitHub (or copy it) to make your plugin's repository.
2. In `Cargo.toml`, set `name` (your plugin's id), `description`, `repository`
   and `authors`. In `src/main.rs`, set the display `name` in `manifest()`.
   Rewrite this README for your plugin's users.
   [Starting from the template](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/plugins/writing-plugins.md#starting-from-the-template)
   has the full list.
3. Build and look at what the recorder will see:
   ```sh
   cargo build
   ./target/debug/call-log --describe
   ```
4. Run it against calls you've already recorded:
   ```sh
   trunk-pro plugin run ./target/debug/call-log ~/TrunkRecorderPro --limit 5
   ```
5. Replace the example with your plugin, and keep the tests passing:
   `cargo test`.
6. Tag a release (`git tag v0.1.0 && git push --tags`); the workflow builds it
   for Linux, macOS and Windows. Then [submit it to the registry](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/plugins/writing-plugins.md#publishing-to-the-registry).

## Documentation

The guide to writing plugins is in Trunk Recorder Pro's docs:

| | |
|---|---|
| [Writing plugins](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/plugins/writing-plugins.md) | The guide: from this template to a plugin in your recorder, how plugins run, settings, audio, testing, releasing, the registry |
| [Plugin events](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/developer/plugins/events.md) | Everything a plugin can subscribe to, field by field |
| [Writing an uploader](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/developer/plugins/uploading.md) | Keys, queues, retries, requests and node health reports: a plugin that sends calls to a service |
| [Plugin protocol](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/developer/plugins/protocol.md) | The wire format and every message, for plugins in other languages |

The SDK's API reference is on [docs.rs/trunk-recorder-plugin](https://docs.rs/trunk-recorder-plugin).

## License

The template is MIT-licensed; your plugin can use any license you like.

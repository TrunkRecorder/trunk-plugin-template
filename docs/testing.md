# Testing

There are three ways to exercise a plugin, from fastest to most real:

1. **Unit tests** drive the plugin in-process with made-up events.
2. **`trunk-lite plugin run`** runs the built plugin against calls you've
   recorded.
3. **Inside the recorder**, with `path` in `plugins.json`.

## Unit tests

The SDK's `testing` module runs a plugin the way the recorder does, without
the recorder:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use trunk_recorder_plugin::{testing, HostMessage, Outcome};

    #[test]
    fn logs_a_call() {
        let dir = testing::temp_dir("my-plugin");
        let mut hello = testing::hello(&dir, json!({ "file": "out.jsonl" }));
        hello.systems[0].config = json!({ "label": "County" });
        let call = testing::call(&dir, "sys1", 101);

        let out = testing::run::<MyPlugin>([HostMessage::Hello(hello), HostMessage::CallConcluded(call.clone())]);

        assert!(out.ready());
        assert_eq!(out.results()[0].1, Outcome::Ok);
    }
}
```

| | |
|---|---|
| `testing::temp_dir(name)` | A fresh, empty folder. |
| `testing::hello(dir, config)` | A `hello` with your settings as JSON, and one P25 system, `sys1` (index 0). Its settings for that system are `hello.systems[0].config`. The data folder is `dir/data`. |
| `testing::call(dir, short_name, tg)` | A 3-second call with its files written under `dir`: the JSON, a silent WAV, and a placeholder M4A in `files.m4a`. Set `files.m4a = None` to test without M4A. |
| `testing::run::<P>(messages)` | Runs the plugin on the messages, then ends its input, so `shutdown` runs, and returns what it said. |
| `out.ready()` | It started: its settings were good. |
| `out.exit_code` | `EXIT_CONFIG` (78) if `start` refused the settings. |
| `out.results()`, `out.logs()`, `out.status()` | What it reported. |

Things worth a test:

- Good settings start the plugin; each kind of bad setting doesn't, with a
  message that says what to fix.
- A call is handled; calls you skip on purpose are reported as `Skipped`.
- No M4A (`files.m4a = None`) is handled the way you decided.
- A failure (a server that's down, say) is reported as `Failed`, not lost.
- Calls in flight are finished or saved at shutdown.

For a plugin that talks to a server, point it at a server your test starts on
`127.0.0.1`. Keep tests off the internet.

## Against your recorded calls

`trunk-lite plugin run` starts your build the way the recorder does and sends
it calls from disk, each as `call.concluded`:

```sh
cargo build
trunk-lite plugin run ./target/debug/my-plugin ~/TrunkRecorderLite --limit 5 --settings settings.json
```

| Option | |
|---|---|
| `<calls…>` | Call `.json` files, or folders. For a folder, the newest `--limit` calls in it (default 10). |
| `--settings file.json` | `{"config": {…}, "systems": {"<short name>": {…}}}` |
| `--capture-dir dir` | The calls' capture folder. Defaults to the recorder's. |
| `--encoder auto\|ffmpeg\|afconvert\|fdkaac\|none` | To test with, or without, M4A. |
| `--grace 30` | Seconds the plugin gets to finish after the last call. |

It prints the plugin's log lines and statuses, and a line for each result:
`✓` done, `–` skipped, `✗` failed.

If you give it an id instead of a path, it runs that installed plugin with its
settings from `plugins.json`.

## By hand

The protocol is JSON lines, so you can also type at a plugin:

```sh
./target/debug/my-plugin
{"type":"hello","api":1,"config":{},"systems":[{"index":0,"short_name":"sys1","kind":"p25"}]}
```

It answers `{"type":"ready"}` if it started. See [Protocol](protocol.md).

## Developing against an unreleased SDK

To try changes to the SDK itself, point Cargo at a local checkout of the
recorder's repository. Create `.cargo/config.toml` (git ignores it):

```toml
[patch.crates-io]
trunk-recorder-plugin = { path = "../trunk-recorder-lite/crates/trunk-recorder-plugin" }
```

Delete the file, then run `cargo update -p trunk-recorder-plugin`, before
committing `Cargo.lock`. Otherwise the lock file records your local path, and
the release build fails.

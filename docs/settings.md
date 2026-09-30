# Settings

A plugin describes its settings as Rust structs. The recorder draws a form from
them, stores what the user enters, and hands it back to the plugin at startup.
You never write form code.

There are two kinds of settings:

- **`Config`**: settings for the plugin as a whole, like a server URL.
- **`SystemConfig`**: settings the form repeats for each system being
  recorded, like an API key per system. Many services give each system its own
  key, so uploaders need this.

Use `NoConfig` for either if you don't need it.

## A settings struct

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct Config {
    /// Server
    ///
    /// Where calls are uploaded. Leave it as it is unless you run your own.
    #[schemars(url)]
    server: String,
    /// Upload encrypted calls
    upload_encrypted: bool,
    /// Retries
    ///
    /// How many times to try a call before giving up.
    #[schemars(range(min = 0, max = 10))]
    retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config { server: "https://api.example.com".into(), upload_encrypted: false, retries: 3 }
    }
}
```

The attributes do the following:

- **`Deserialize`** reads what the user entered.
- **`JsonSchema`** describes the struct to the form. It comes from the
  [`schemars`](https://docs.rs/schemars/1) crate, version 1.
- **`Serialize`** lets the schema include your defaults, so the form starts
  filled in with them. Without it, fields start empty.
- **`#[serde(default)]`** lets the user leave any field empty. The field then
  takes its value from `Default`. Leave it out and every field becomes
  required, and an existing install breaks the day you add a field.
- **`#[serde(rename_all = "camelCase")]`** isn't required, but it's the
  recorder's convention for its own settings.

Doc comments become the form's text. The first paragraph is the field's
**label**; the rest is **help** shown under it. Wrap them however you like: line
breaks inside a paragraph are joined up. Keep labels short, and write help
for someone who isn't a programmer. The doc comment on the struct itself is
for programmers, and isn't shown.

## What the form can show

| Rust | Form |
|---|---|
| `String` | Text box |
| `String` with `#[schemars(extend("x-secret" = true))]` | Password box (hidden as it's typed, and never logged) |
| `String` with `#[schemars(url)]` | Text box that checks for a URL |
| `String` with `#[schemars(extend("x-multiline" = true))]` | Text area |
| `bool` | Switch |
| `u32`, `i64`, `f64`, … with `#[schemars(range(min = …, max = …))]` | Number box |
| An enum of unit variants | Menu. A variant's doc comment is its label. |
| `Vec<String>`, `Vec<u32>`, … | List |
| `Option<T>` | Same as `T`. Empty means `None`. |
| A struct | A group of its fields |

Fields appear in the order you declare them.

The form doesn't handle maps, lists of structs, or enums that carry data. Keep
settings flat, and name things users know, like "API key" rather than
"auth token".

For a menu:

```rust
#[derive(Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
enum Quality {
    /// Low (smaller files)
    Low,
    #[default]
    /// Normal
    Normal,
}
```

## Settings for each system

```rust
#[derive(Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
struct SystemConfig {
    /// API key
    #[schemars(extend("x-secret" = true))]
    api_key: String,
}
```

In `start`, `setup.systems` lists every system, each with its index, short
name, kind (`p25`, `smartnet` or `conventional`), and your `SystemConfig`.
The config is `None` when the user left that system's settings empty.
Events name systems by index (`call.system`), so build your lookup in `start`:

```rust
let keys: HashMap<u16, String> = setup
    .systems
    .iter()
    .filter_map(|s| Some((s.index, s.config.as_ref()?.api_key.clone())))
    .filter(|(_, k)| !k.is_empty())
    .collect();
```

Conventional channels count as one system, with index
`trunk_recorder_plugin::CONVENTIONAL` (65535).

## Checking settings

The SDK rejects settings that don't parse, such as text where a number goes,
before `start` runs. Check everything else in `start`, and return an `Err`
that tells the user what to fix:

```rust
if keys.is_empty() {
    return Err("Add an API key for at least one system".into());
}
```

The recorder shows the error, and doesn't start the plugin again until
recording next starts.

## Seeing the form

The recorder's **Plugins** page draws the form. Add your build there (see
[Getting started](getting-started.md#6-run-it-inside-your-recorder)) and press
**Settings** to see it the way users will: fields in declaration order, labels
and help from your doc comments, defaults as placeholder text, secrets hidden
behind **Show**, and the `SystemConfig` fields repeated under each system's
short name. Empty text fields aren't saved, so your defaults apply.

## Where settings live

The recorder keeps plugin settings in `plugins.json`, next to its
`config.json`:

```json
{
  "plugins": {
    "my-plugin": {
      "enabled": true,
      "config": { "server": "https://api.example.com", "retries": 3 },
      "systems": { "county": { "apiKey": "…" } }
    }
  }
}
```

Settings for each system are keyed by short name, so they stay with the
system when systems are added or reordered.

## Changing your settings later

Users upgrade plugins but keep their settings, so:

- **Adding a field** is safe, as long as the struct has `#[serde(default)]`.
- **Renaming a field** loses what users entered. Use `#[serde(alias = "old")]`
  to keep reading the old name. Aliases also let people coming from Trunk
  Recorder paste their old plugin settings: the OpenMHz plugin reads
  `uploadServer` and `openmhzSystemId`, TR's names, as well as its own.
- **Removing a field** is safe, since unknown fields are ignored.
- **Changing a field's type** breaks existing settings. Add a new field instead.

## Seeing the schema

`./target/debug/my-plugin --describe` prints the manifest, which includes the
schemas the form is drawn from. They use a small subset of
[JSON Schema](https://json-schema.org). [Protocol](protocol.md#settings-schemas)
lists it, for plugins that write their schemas by hand.

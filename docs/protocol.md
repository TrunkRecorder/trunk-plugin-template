# Protocol

This is the wire protocol between the recorder and a plugin, for writing a
plugin in a language other than Rust. The Rust SDK implements all of it. The
Rust types in [`protocol.rs`](https://docs.rs/trunk-recorder-plugin/latest/trunk_recorder_plugin/protocol/index.html)
are the reference.

This is plugin API version **1**.

## Transport

- The recorder runs the plugin's executable **with no arguments**, with its
  working directory set to the plugin's data folder.
- The recorder writes to the plugin's **stdin**, and reads the plugin's
  **stdout**. Each message is one JSON object on one line, UTF-8, ending in
  `\n`.
- The plugin's **stderr** is read line by line into the recorder's log.
- **Both sides ignore** message types and fields they don't know. New fields
  and types can appear within an API version; removing or changing one needs a
  new version.

## `--describe`

Run with the single argument `--describe`, a plugin prints its manifest as one
JSON object on stdout and exits 0:

```json
{
  "id": "openmhz",
  "name": "OpenMHz",
  "version": "1.0.0",
  "description": "Uploads calls to OpenMHz.",
  "api": 1,
  "subscribe": ["call.concluded"],
  "audio_formats": ["m4a"],
  "config": { "type": "object", "properties": { … }, "x-order": [ … ] },
  "system_config": { "type": "object", "properties": { … }, "x-order": [ … ] },
  "system_overrides": ["server"],
  "site_overrides": ["apiKey"],
  "repository": "https://github.com/TrunkRecorder/trunk-plugin-openmhz",
  "authors": ["…"],
  "license": "GPL-3.0-or-later"
}
```

| Field | Required | |
|---|---|---|
| `id` | yes | `[a-z0-9][a-z0-9-]*` |
| `name`, `version`, `description` | yes | |
| `api` | yes | The plugin API version it speaks. The recorder refuses a higher one than it knows. |
| `subscribe` | yes | Topics: `call.start`, `call.update`, `call.end`, `call.concluded`, `unit`, `audio`, `voice`, `status`, `totals`, `tone_pattern`, `log`. |
| `audio_formats` | no | `["m4a"]` to get M4A with `call.concluded`; add `"wav"` to have the WAV written as well. |
| `config`, `system_config` | no | [Settings schemas](#settings-schemas). Leave them out for none. |
| `system_overrides` | no | Keys of `config` a system may set again for itself (another server, say). None: every system has the plugin's settings. |
| `site_overrides` | no | Keys of `system_config` a site of a system may set again for itself (an account for that site). None: a system's sites have its settings. |
| `homepage`, `repository`, `authors`, `license` | no | |
| `page` | no | `{ "title": "…" }`: a page of its own in the recorder's navigation, for settings too many for its card |
| `contribute` | no | Buttons elsewhere in the interface: `[{ "on": "signal", "label", "hint", "append", "item" }]` adds `item` to the settings list `append` from each of a system's signals |
| `live_settings` | no | `true`: it takes new settings as it runs (`settings`, below) instead of being restarted |

It must finish within 10 seconds.

## Recorder → plugin

Every message has a `type`. The first is always `hello`.

### `hello`

```json
{
  "type": "hello",
  "api": 1,
  "host": { "name": "trunk-pro", "version": "0.5.0" },
  "config": { "server": "https://api.openmhz.com" },
  "systems": [
    { "index": 0, "id": 1, "short_name": "county", "kind": "p25", "name": "County Public Safety",
      "fcc_call_sign": "WPXT459", "fcc_name": "COUNTY OF EXAMPLE",
      "fcc": { "851012500": { "call_sign": "WPXT459", "name": "COUNTY OF EXAMPLE" } },
      "sites": [{ "id": 1, "name": "north" }, { "id": 4, "name": "south", "config": { "apiKey": "…" } }],
      "config": { "apiKey": "…" } },
    { "index": 1, "id": 2, "short_name": "city", "kind": "p25", "sites": [{ "id": 2, "name": "city" }],
      "config": { "apiKey": "…" }, "settings": { "server": "https://openmhz.example.org" } },
    { "index": 2, "id": 3, "short_name": "conv", "kind": "conventional", "config": null }
  ],
  "capture_dir": "/home/me/TrunkRecorderPro",
  "data_dir": "/home/me/.config/trunk-pro/plugin-data/openmhz",
  "audio_formats": ["wav", "m4a"]
}
```

- **`config`** holds the plugin's settings as the user entered them. It's
  `null` when there are none.
- **`systems`** lists the systems calls can come from, conventional ones
  included, but not those the user turned the plugin off for: nothing about
  those is sent. A system's **`id`** is its identity, the same across runs
  and renames; events carry it as `system_id`. Its **`index`** is the
  `system` events carry, for this run only. **`kind`** is `p25`, `smartnet`,
  `dmr`, `nxdn`, `edacs` or `conventional`. **`name`** (what people call it)
  and the FCC licenses (**`fcc`**, by frequency, and the call sign and
  licensee that name it, **`fcc_call_sign`** and **`fcc_name`**) are optional
  and often missing.
- A system's **`config`** holds the plugin's settings for that system, or is
  `null`.
- A system's **`settings`**, when it's there, is the plugin's `config` with
  the changes that system makes to the keys the manifest's `system_overrides`
  names. Without it, the plugin's `config` applies to the system as it is.
- A system's **`sites`** (trunked systems only, and not those the plugin is
  off for) each have an **`id`**, permanent as a system's (events carry it as
  `site_id`), the **`name`** events carry as `site`, and, when the site sets
  some of the keys `site_overrides` names, **`config`**: the system's `config`
  with those changes.
- **`audio_formats`** lists what `call.concluded` will carry: always `wav`,
  plus the formats the plugin asked for that the recorder can make.

The plugin answers `ready` when it's running. If it can't run with these
settings, it sends a `status` of `error` saying why, and exits with status
**78**.

### Events

`call.concluded`, `call.start`, `call.update`, `call.end`, `unit`, `audio`,
`voice`, `status`, `totals`, `tone_pattern` and `log`. Each message is the
event's fields plus `type`. See [Events](events.md).

### `action`

```json
{ "type": "action", "request": 7, "action": "test", "value": { "name": "Discord", "url": "…" } }
```

Someone pressed one of the plugin's buttons (`x-actions` in its settings
schema). `value` is the settings object the button is on, as edited (saved or
not). Answer with `action.result` and the same `request` within 15 seconds;
the answer is shown by the button, in the browser that pressed it.

### `settings`

The plugin's settings changed while it runs: the `hello` it would start with
now, with `"type": "settings"`. Only a plugin with `live_settings` is sent
one; others are restarted. A plugin that can't take them reports an `error`
status and keeps the ones it had.

### `calls`

```json
{ "type": "calls", "request": 3, "entries": [{ "path": "county/2026/9/30/101-1790771550_857587500", "record": { "call_num": 412, "short_name": "county", "talkgroup": 101, … }, "audio": true, "json": true }], "next": "1790771550000:county/2026/9/30/101-1790771550_857587500" }
```

The answer to a [`calls.query`](#callsquery), with its `request`: a page of
recorded calls, newest first. Each entry has the call's `path`, its `record`
(the call JSON), and whether its audio and `.json` files are still kept.
**`next`**, when there are more, is the `cursor` for the next page. **`error`**
says why the answer is only the recent calls (the recorder's call store is
off).

### `settings.saved`

```json
{ "type": "settings.saved", "request": 4, "ok": false, "message": "saving would restart it: its systems have settings of their own" }
```

The answer to a [`settings.save`](#settingssave), with its `request`: saved,
or `message` saying why not. Saved, the new settings came first, as
`settings`.

### `shutdown`

```json
{ "type": "shutdown", "grace_s": 10 }
```

The recorder is stopping. Finish or save what's in progress, then exit. The
recorder closes stdin after this message. It kills the process if it's still
running `grace_s` seconds later. Treat stdin closing without a `shutdown` the
same way.

## Plugin → recorder

### `ready`

```json
{ "type": "ready" }
```

### `log`

```json
{ "type": "log", "level": "info", "message": "Uploading calls for county" }
```

`level` is `error`, `warn`, `info` or `debug`. The recorder drops `debug`.

### `status`

```json
{ "type": "status", "state": "warning", "message": "OpenMHz isn't answering; 12 calls waiting" }
```

`state` is `ok`, `warning` or `error`. It's shown with the plugin until the
next status.

### `call.result`

```json
{ "type": "call.result", "path": "county/2026/9/30/101-1790771550_857587500", "outcome": "ok", "message": "", "url": "https://openmhz.com/system/county?call=…", "took_ms": 840 }
```

`path` is the `path` of the `call.concluded`. `outcome` is `ok`, `skipped`
(not handled, on purpose) or `failed` (given up on). `message` and `url` are
optional. So is `took_ms`, how long the work on the call took (an upload's
last try): with it, the recorder's dashboard tells waiting from uploading.

Send one for every call: the recorder keeps a call's files until every plugin
taking recorded calls has reported on it, and counts a call not reported on
after an hour as failed.

### `metrics`

```json
{ "type": "metrics", "queued": 12, "retrying": 12, "inFlight": 0, "retries": 30, "bytesSent": 5242880, "latencyMs": 640.0,
  "lastOk": 1790771500.0, "lastError": 1790771555.0, "lastErrorText": "HTTP 503",
  "endpoints": [{ "name": "OpenMHz", "state": "down", "lastError": "HTTP 503" }], "extra": { "listeners": 3 } }
```

How the plugin's work is going, for the recorder's dashboard. Every field is
optional; totals are since the plugin started. An endpoint's `state` is
`unknown`, `up`, `degraded` (answering, with failures) or `down`. `extra` is
shown as it is. Send it when something changes, at most every few seconds.
(The field names here are camelCase, unlike the rest of the protocol.)

### `calls.query`

```json
{ "type": "calls.query", "request": 3, "system": "county", "talkgroup": 101, "limit": 10 }
```

Recorded calls, asked of the recorder: answered with [`calls`](#calls) and the
same `request`. Every field but `request` is optional, and each narrows the
calls: `system` (a short name), `talkgroup`, `unit` (calls that radio talked
on), `from` and `to` (Unix ms: started at or after, and before), `cursor` (the
last page's `next`) and `limit` (1 to 500; 100 when left out). Ask this way
rather than opening the recorder's database: its tables change between
versions.

### `settings.save`

```json
{ "type": "settings.save", "request": 4, "settings": { "serverName": "County Fire" } }
```

Save some of the plugin's own settings, as if typed in on its form: each key
of `settings` replaces that setting (`null` removes it), and the rest stay.
The recorder writes them to its config, sends them back as `settings`, and
answers with [`settings.saved`](#settingssaved). It refuses when the save
would restart the plugin: the plugin has to have `live_settings`, run as one
copy, have no per-system settings, and keep every setting it needs set. For
what the plugin learns that the user would otherwise type (mark those fields
`readOnly`); its own state goes in its data folder.

### `action.result`

```json
{ "type": "action.result", "request": 7, "ok": true, "message": "Sent a test to Discord" }
```

The answer to an `action`, with its `request`.

A line that isn't JSON is logged as it is.

## Exit status

| Status | Meaning | The recorder |
|---|---|---|
| 0 | Stopped as asked | Nothing. |
| 78 | Can't run with these settings | Shows the error, and doesn't restart it until its settings change or recording next starts. |
| Anything else, or a signal | Crashed | Restarts it after 1, 2, 4, … up to 60 seconds. |

## Delivery

The recorder queues 1024 messages for each plugin, and drops new ones while
the queue is full, except a call's `call.start` and `call.end` and the answers
to what the plugin asked (`calls`, `settings.saved`): those are kept and sent,
in order, once there's room. (A call that starts and ends while the plugin is
behind isn't sent at all.) Messages
 queued while a plugin restarts
are delivered to the new process, after its `hello`.

## Settings schemas

`config` and `system_config` are [JSON Schema](https://json-schema.org)
objects. The recorder's settings form understands this subset:

| Schema | Form |
|---|---|
| `{"type": "object", "properties": {…}, "x-order": [keys]}` | A group of fields, in `x-order` (else in key order) |
| `{"type": "string"}` | Text box |
| `…, "x-secret": true` | Password box |
| `…, "format": "uri"` | Text box that checks for a URL |
| `…, "x-multiline": true` | Text area |
| `{"type": "integer"}`, `{"type": "number"}` with `minimum`, `maximum` | Number box |
| `{"type": "boolean"}` | Switch |
| `{"type": "string", "enum": [...], "x-enum-labels": [...]}` | Menu |
| `{"type": "array", "items": {"type": "string"}}` (or number) | List |
| `{"type": "array", "items": {"type": "object", …}}` | A list of groups, added and removed one by one |
| `{"type": "string", "x-system": true}` | Menu of the recorder's systems (short names); follows renames |
| `{"type": "array", "items": {"enum": [...]}}` | The choices, to tick |
| `x-systems`, `x-signals`, `x-talkgroups`, `x-ref`, `x-tones`, `x-title`, `x-summary`, `x-when`, `x-actions`, `x-slider`, `x-open`, `readOnly` | Systems or signals to tick, talkgroups typed in and checked, items of another list, tones typed in, list items titled and summed up, fields shown only when they apply, buttons, sliders, fields the plugin fills in itself: see [the recorder's guide](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/plugins/writing-plugins.md#settings-and-the-form) |


On any field: `title` (the label; the key if there's none), `description`
(help text), `default`.

On an object: `required`, the keys that have to be filled in. Until they are,
the recorder shows the plugin (for `config`) or that system (for
`system_config`) as not set up. `"x-required": true` on a field means the same.

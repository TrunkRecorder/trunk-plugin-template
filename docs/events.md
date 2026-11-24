# Events

A plugin gets only the topics listed in its manifest's `subscribe`. The
recorder doesn't build events nobody subscribed to, so subscribing to a topic
you don't use costs everyone something.

| Topic | Method | When | Rate |
|---|---|---|---|
| `call.concluded` | `call_concluded` | A recorded call's files are on disk | One per call |
| `call.start` | `call_start` | A call starts (recorded or not) | One per call |
| `call.update` | `call_update` | A call changes while it runs: another radio talking, an emergency, a patch, … | As it changes |
| `call.end` | `call_end` | A call ends | One per call |
| `unit` | `unit` | A radio registers, affiliates, … | Busy systems: many per second |
| `audio` | `audio` | Live audio of recording calls | About 50 per second per call |
| `voice` | `voice` | Live voice of recording digital calls, as vocoder frames | One beside each `audio` chunk |
| `status` | `status` | Every site's state | Every 5 seconds |
| `totals` | `totals` | Every series the recorder measures, as running totals | Every 10 seconds |
| `tone_pattern` | `tone_pattern` | A call's tones made a pattern: a page, an alert tone, a DTMF or KNOX string, a Morse ID | As heard |
| `log` | `log_line` | A line the recorder logged | Each line |

The examples below are the JSON on the wire. The Rust SDK hands you the same
data as structs; the field names match. Times are Unix seconds. Frequencies
are in hertz.

Events name their system three ways. **`system_id`** is its identity: the
same across runs and renames, never another system's. Store anything by it.
**`short_name`** is what users know it by, and changes when it's renamed.
**`system`** is its number for this run only. Calls and unit events also carry
the **`site`** they were heard on (its name) and its **`site_id`**, the site's
identity as `system_id` is the system's.


## `call.concluded`

A recorded call is finished and its files are written. This is the event for
uploaders and archivers.

```json
{
  "type": "call.concluded",
  "path": "county/2026/9/30/101-1790771550_857587500",
  "system": 0,
  "call": {
    "call_num": 412, "short_name": "county", "system_id": 1, "site": "county", "site_id": 1,
    "sites": [{ "id": 1, "name": "county" }], "talkgroup": 101,
    "talkgroup_tag": "Fire Dispatch", "talkgroup_description": "", "talkgroup_group": "Fire", "talkgroup_group_tag": "Fire Dispatch",
    "freq": 857587500, "start_time": 1790771550, "stop_time": 1790771554, "call_length": 4, "call_length_ms": 4180,
    "emergency": 0, "encrypted": 0, "priority": 0, "phase2_tdma": 0, "tdma_slot": 0, "audio_type": "digital",
    "freqList": [{ "freq": 857587500, "time": 1790771550, "pos": 0, "len": 4, "error_count": 0, "spike_count": 0 }],
    "srcList": [{ "src": 1116707, "time": 1790771550, "pos": 0, "emergency": 0, "signal_system": "", "tag": "", "tag_ota": "E14" }]
  },
  "files": {
    "json": "/home/me/TrunkRecorderPro/county/2026/9/30/101-1790771550_857587500.json",
    "wav": "/home/me/TrunkRecorderPro/county/2026/9/30/101-1790771550_857587500.wav",
    "m4a": "/home/me/TrunkRecorderPro/county/2026/9/30/101-1790771550_857587500.m4a"
  }
}
```

- **`path`** is the call's key: its location relative to the capture folder,
  without an extension. Report results against it with `host.call_result(&call.path, …)`.
- **`system`** is the system's number for this run (`SystemInfo.index`). Know
  systems by **`call.system_id`** instead: that is the system's identity, the
  same from run to run and through renames. `call.short_name` is what users
  know it by, and changes when it's renamed.
- **`call`** is the call's JSON file, in
  [Trunk Recorder's format](https://trunkrecorder.com/docs/notes/CALLFILE),
  so existing tools and services understand it. The SDK's `CallRecord` names
  every field the recorder writes; fields it doesn't know (from another
  recorder's file) are in `call.extra`. `call.error_count()` and
  `call.spike_count()` add up the per-frequency counts.
- **How long it is**: `call.audio_ms()`, from `call_length_ms`.
  `call_length` is whole seconds, as Trunk Recorder writes it, so a 0.4 s call
  says 0. `call.has_audio()` is `false` for a call kept with no audio (an
  encrypted one, or with the recorder's **Keep calls with no audio** on):
  there's nothing to play or upload.
- **`call.patched_talkgroups`** is there only when the call's talkgroup was
  patched with others during the call: every talkgroup in the patch, its own
  included, in ascending order (`CallRecord::patched_talkgroups`, empty when
  absent). A patched call is usually recorded on the patch's supergroup, which
  is often not in the talkgroup list, so this is how to tell what it carried.
- **`files.m4a`** is there only if the plugin asked for M4A and the recorder
  could make it. With an M4A, the WAV may not have been written. See
  [Audio](audio.md).

Only recorded calls conclude. Calls the recorder skipped (encrypted, no free
recorder, an unknown talkgroup when those aren't recorded) have no files.

A system can have several sites, each a control channel the recorder follows.
A call heard on more than one is recorded on each and the best copy saved:
`short_name` is the system, and **`site`** is the site the call was kept from
(a system of one site: its short name; none for a conventional channel's call),
with its permanent **`site_id`**. **`sites`** lists every site the call was
recorded on (`id` and `name`), the kept copy's first. A copy the saved one doesn't hold all of is saved too (better a duplicate than
a transmission lost), and copies of one transmission share
**`transmission_id`**. A system with `conversationMode` off makes each
transmission a call of its own.

## `call.start`, `call.update` and `call.end`

A call started, changed, or ended. Unlike `call.concluded`, these cover calls
that aren't recorded too. All three carry the whole call as it is at that
moment (`CallInfo`). `call.update` comes whenever any of it changes while it
runs: another radio talking (`sources` gains it), an emergency, encryption,
its patched talkgroups, whether it's recorded, its key or its decryption, a
radio's position; on a conventional channel, its talkgroup from the air or its
tone. `call.end` comes as soon as the call ends; its files, if it was
recorded, come later in `call.concluded`.

```json
{
  "type": "call.start",
  "id": 412, "system": 0, "system_id": 1, "short_name": "county", "site": "county", "site_id": 1,
  "talkgroup": 101, "talkgroup_tag": "Fire Dispatch",
  "freq_hz": 857587500, "tdma_slot": null,
  "analog": false, "encrypted": false, "algid": null, "key_id": null, "decrypted": false,
  "emergency": false, "duplex": false, "packet": false, "priority": 4,
  "recording": true, "reason": null,
  "start_time": 1790771550.2,
  "units": [1116707],
  "sources": [{ "unit": 1116707, "tag": "Engine 12", "time": 1790771550.2, "emergency": false, "encrypted": false, "duplex": false, "packet": false, "priority": 4 }],
  "patched_talkgroups": [101, 102],
  "patched": [{ "talkgroup": 102, "alpha_tag": "Fire Tac 2" }]
}
```

- **`id`** is unique while the recorder runs. It's the same number as
  `call_num` in the concluded call, except for a call heard on several sites:
  it starts once, when its first copy begins, and ends once, when its last
  copy ends, under the first copy's `id`. Its `site` and `freq_hz` are the
  copy speaking for it (a `call.update` says when that changes), and the copy
  that concludes may be another site's, with another number. Match those by
  system, talkgroup and time if you need to.
- **`recording`**: `false` means the call isn't recorded, and **`reason`**
  says why: `encrypted`, `unknown_tg` (not in the talkgroup list, and unknown
  talkgroups aren't recorded), `no_recorder` (all in use) or `no_source` (no
  radio covers its frequency). More reasons may be added.
- **`units`**: radios heard on the call so far. At `call.end`, all of them.
  **`sources`** has the same radios with each one's name (`tag`, as the call
  JSON's `srcList` has it) and what its transmission said of itself.
- **`patched_talkgroups`**: the talkgroups patched with this one so far, its
  own included, ascending; empty when it isn't patched. **`patched`** has the
  others, without its own (which `talkgroup_tag` names), each with its
  `alpha_tag` from the system's talkgroups (left out when it has none).
- **`duplex`**, **`packet`** and **`priority`** are the grant's service
  options (priority 1 lowest, 4 the default, 7 highest; 0 not given).
  **`algid`** and **`key_id`** are the cipher an encrypted P25 call's voice
  named, once heard; **`decrypted`** says some of it was decrypted with the
  system's keys.
- **`talkgroup_ota`**, on a conventional P25, DMR or NXDN channel, is the
  talkgroup the air named, when a channel row files the call under another
  `talkgroup`.

## `unit`

A radio did something on the control channel.

```json
{ "type": "unit", "system": 0, "system_id": 1, "short_name": "county", "site": "county", "site_id": 1, "kind": "affiliation", "unit": 1116707, "tag": "Engine 12", "talkgroup": 101, "alpha_tag": "Fire Dispatch", "emergency": false, "time": 1790771551.4 }
```

`tag` is the radio's name and `alpha_tag` the talkgroup's, looked up as the
event is sent (left out when there's none). `emergency` is set when the
message said it's an emergency (an emergency alarm, a data grant's options).

`kind` is one of:

| Kind | Meaning | `talkgroup` |
|---|---|---|
| `registration` | The radio joined the system | none |
| `deregistration` | It left (switched off, out of range) | none |
| `affiliation` | It selected a talkgroup | the talkgroup |
| `acknowledge` | It acknowledged the system | none |
| `location` | It reported its location | the talkgroup |
| `data_grant` | It was granted a data channel | none |
| `answer_request` | Another radio asked to call it | the talkgroup |
| `call_alert` | It was paged | the talkgroup |
| `emergency_alarm` | It sent an emergency alarm | the talkgroup |

P25 systems report all of these. SmartNet systems report fewer.

## `audio`

Live audio of a call being recorded, as it's decoded: 16-bit mono PCM,
little-endian, base64-encoded. `chunk.samples()` decodes it.

```json
{ "type": "audio", "call_id": 412, "system": 0, "system_id": 1, "short_name": "county", "talkgroup": 101, "sample_rate": 8000, "pcm": "AAABAP7/…" }
```

Chunks of one call come in order. Chunks of different calls are interleaved.
Use `call_id` to tell them apart, and `call.start` / `call.end` (subscribe to
them too) to know when a call's audio begins and ends. A call heard on several
sites plays from whichever site's copy has it at the moment, so a chunk can
carry the `call_id` of a copy that had no `call.start`: it is the call of the
same system and talkgroup that did.

Live audio is only for plugins that get the call as it happens. A plugin the
user delays calls for (`pluginDelays`) isn't sent it, and the audio of a call
decrypted with the system's keys goes only to plugins allowed decrypted calls
(`shareDecrypted`).

This topic is busy. Hand the audio to your own thread straight away.

## `voice`

The vocoder frames each `audio` chunk of a recording digital call was made
from, FEC-corrected and in order, sent beside it: for a plugin that relays
digital voice (to a hotspot network, say) without decoding it to audio and
encoding it again.

```json
{
  "type": "voice", "call_id": 412, "system": 0, "system_id": 1, "short_name": "county", "talkgroup": 101,
  "unit": 1116707, "unit_name": "Engine 12", "decrypted": false,
  "frames": [{ "codec": "imbe", "bits": "a3f0…", "e0": 0, "errs": 1, "erased": false, "kind": "voice" }]
}
```

- **`codec`** is `imbe` (P25 Phase 1, ProVoice: 88 bits) or `ambe` (AMBE+2
  half rate: P25 Phase 2, DMR, NXDN: 49 bits).
- **`bits`** are the frame's information bits as hex, first bit first, the
  last byte padded with zeros. `VoiceFrame` has them as a list of 0s and 1s.
- **`errs`** is the bit errors FEC corrected in the frame, and **`e0`**
  IMBE's in u0, the word carrying pitch. **`erased`** means the frame was lost.
- **`kind`** is what the vocoder made of it: `voice`, `repeat`, `muted`,
  `erasure` or `tone` (with **`tone`**: the tone's ID and level).

Voice goes where live audio goes: not to a plugin delayed for the call,
decrypted only to plugins allowed decrypted calls, and encrypted voice the
recorder can't decrypt not at all. A frame is 20 ms; this topic is as busy as
`audio`.

## `status`

Every site's state, every 5 seconds: an entry for each site, so a system of
several sites has several, with the same `index`, `system_id` and
`short_name` and its own `site_name` and `site_id`.

```json
{
  "type": "status", "time": 1790771560.0,
  "systems": [
    { "index": 0, "system_id": 1, "short_name": "county", "site_name": "county", "site_id": 1,
      "key": "sys/county", "protocol": "p25", "control_channel_hz": 851012500, "source": 0,
      "identity": { "nac": 928, "wacn": 781824, "sysId": 939, "rfss": 1, "site": 5 },
      "decode_rate": 38.5, "active_calls": 3, "recording": 2 }
  ]
}
```

- **`control_channel_hz`** is `null` while the system searches for its
  control channel. **`source`** is the radio it's heard on.
- **`decode_rate`** is control channel messages decoded per second. A healthy
  P25 control channel gives about 40.
- **`identity`** is what the site's control channel says it is, each field
  once it has: P25 `nac`, `wacn`, `sysId`, `rfss`, `site`; SmartNet `sysId`
  and `site`; DMR `colorCode`, Tier III also `network` and `site`; NXDN `ran`,
  Type-C also `sysId` and `site`.
- **`key`** is where the site's series are in `totals`.

## `totals`

Every series the recorder measures (the ones its dashboard charts), as running
totals since recording started, every 10 seconds, with the sites, conventional
channels and radios they're of (`Totals`):

```json
{
  "type": "totals", "time": 1790771560.0, "since": 1790771000.0,
  "series": {
    "sys/county/cc/good": { "sum": 21400, "s": 560, "now": 39 },
    "sys/county/calls/recorded": { "sum": 212, "s": 190 }
  },
  "sites": [{ "index": 0, "system_id": 1, "short_name": "county", "site_name": "county", "site_id": 1, "key": "sys/county", "protocol": "p25", "control_channel_hz": 851012500, "identity": {} }],
  "channels": [],
  "sources": [{ "key": "src/800", "index": 0, "name": "800", "driver": "airspy", "auto_tune": false }]
}
```

A counter's `sum` is what it counted and `s` the seconds in which it counted
any; a gauge's `sum` is its value times the seconds it held it, and `s` those
seconds. `now` is the latest value. Take the difference of two messages for a
window of your own (`totals.since(Some(&earlier))`); a later `since` means
recording restarted, and the totals with it. `clock`, when it's there, says
how the recorder's clock is kept (`sync`: `ntp`, `gps`, …) and how far it is
from a P25 system's broadcast time (`system_offset_ms`). The keys are listed in
[the recorder's guide](https://github.com/TrunkRecorder/trunk-recorder-pro/blob/main/docs/plugins/writing-plugins.md#series).

The SDK's `health` module makes node health reports for a call-collecting
service of these: see [Writing an uploader](uploading.md#node-health-reports).

## `tone_pattern`

A call's in-band tones made a pattern, as the call JSON's `tone_patterns` will
list it. The pattern's fields are in the event itself, beside the call's.

```json
{
  "type": "tone_pattern", "short_name": "county", "system_id": 1, "site": "county", "call_num": 412,
  "talkgroup": 101, "alpha_tag": "Fire Dispatch", "src_tag": "Station 1", "time": 1790771552.0,
  "pattern": "two_tone", "tones": [849.63, 1292.85], "lengths": [1.0, 3.04],
  "format": "Quick Call II", "pos": 0.4, "len": 4.04, "src": 1116707, "origin": "frame",
  "text": "two-tone 849.63 → 1292.85 Hz, 1 + 3.04 s (Quick Call II)", "signals": ["FD_PAGE"]
}
```

`pattern` is `two_tone` (a page), `long`, `hi_low` (a warble), `pulsed`,
`digits` (DTMF, with `digits`; `knox: true` for KNOX), `morse`, `sequence` or
`beep`. `signals` names the system's signals it matches. `decrypted` is set
when it was heard in voice decrypted with the system's keys (only plugins
allowed decrypted calls get those). The SDK has the pattern in
`TonePatternEvent.pattern`.

## `log`

Each line the recorder logs, at the level its log is set to:

```json
{ "type": "log", "time": 1790771552.1, "level": "info", "system": "county", "text": "[county]\t412C\tTG: 101 …", "line": "[2026-09-30 14:02:11.123456] (info)   [county]\t412C\tTG: 101 …" }
```

`level` is `trace`, `debug`, `info`, `warning`, `error` or `fatal`. `system`
is the system's short name, when the line is about one. `text` is the line
after its time and level; `line` is the whole line as the console prints it,
without colour. It's for passing the log on (to MQTT, a chat). A plugin isn't
sent its own lines. Don't log something for each line you get: two plugins
that pass lines on would keep each other busy.

## Systems and sites a plugin is off for

The user can turn a plugin off for a system, or for one site of a system.
Nothing about a system it's off for reaches it: it isn't in `hello`, and no
events, `status` entries or `totals` series of it are sent (the radios', shared
by systems, still are). A site it's off for isn't among its system's `sites`,
and its unit events, `status` entry and `totals` series aren't sent, nor a
recorded call only it heard. Calls as they go (`call.start`, `call.update`,
`call.end`, live audio) are the system's, and still come. Don't count on
hearing of every system or site.

## Order and loss

Events of one topic arrive in the order they happened. `call.concluded` comes
after `call.end`, sometimes by a moment longer when M4A is being made. The user
can delay a plugin's calls (`pluginDelays`, by talkgroup): its call events then
come that much later, and its live audio and voice not at all.

A plugin that falls behind loses events (see
[How plugins work](how-plugins-work.md#one-thread-return-quickly)), but not a
call's `call.start` or `call.end`, nor an answer to something it asked: those
wait their turn. (A call that starts and ends while the plugin is behind isn't
told at all.)
 Still, time out calls you track: a plugin restarted in the
middle of a call gets its `call.end` without its `call.start`.

## Topics may grow

New fields and new topics may appear in later versions of the recorder.
Ignore what you don't know; the SDK does this for you.

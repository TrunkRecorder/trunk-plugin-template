# Audio

Every recorded call is a **WAV** file: 16-bit mono, 8 kHz. That's what the
recorder decodes, so it's lossless, and it's always there.

Many services want **M4A** (AAC audio in an MP4 file) instead, which is about
a tenth of the size. OpenMHz and Broadcastify Calls both do.

## Asking for M4A

List it in your manifest:

```rust
Manifest {
    subscribe: vec![topic::CALL_CONCLUDED.into()],
    audio_formats: vec![format::M4A.into()],
    ..trunk_recorder_plugin::manifest!()
}
```

The recorder then encodes each call once, whichever and however many plugins
asked, and puts the file next to the WAV. `call.concluded` gives its path in
`files.m4a`. Encoding happens off the recorder's main thread, so M4A calls
reach you a moment later than WAV-only calls would.

## M4A might not be there

The recorder doesn't include an AAC encoder. It uses one already on the
computer, in this order:

1. **ffmpeg**, on any platform. It encodes at Trunk Recorder's settings:
   AAC-LC, 16 kHz, 32 kbps.
2. **afconvert**, which is part of macOS.
3. **fdkaac**.

If none is there, or the user turned M4A off, calls come as WAV only. Your
plugin has to cope. You find out in two places:

- At startup: `setup.has_format(format::M4A)` is `false`. This is the place to
  tell the user, once:
  ```rust
  if !setup.has_format(format::M4A) {
      host.warn("No M4A encoder: uploading WAV (install ffmpeg for smaller uploads)");
  }
  ```
- For each call: `call.files.m4a` is `None`. Encoding a single call can fail
  even when an encoder is there.

What to do without M4A depends on the service. Upload the WAV if it accepts
WAV. If it doesn't, report the call as `Failed` and explain, or set a
`Warning` status once so the user knows to install ffmpeg.

On macOS, afconvert is always there, so M4A always is.

## Encoder settings

The user can pick the encoder (or none) and the bitrate in `plugins.json`:

```json
{ "audio": { "encoder": "auto", "bitrateKbps": 32 } }
```

`encoder` is one of `auto`, `ffmpeg`, `afconvert`, `fdkaac` or `none`.
`trunk-lite plugin list` shows which encoder was found.

## Files are shared

The WAV and M4A files belong to the recorder: its call history plays them,
and other plugins read them. Don't move, change or delete them. If you need to
change the audio (normalize it, trim it), write your own copy in your data
folder or a temporary file.

## Live audio

Plugins that stream live audio subscribe to the `audio` topic instead. It's
raw PCM (see [Events](events.md#audio)); encoding it for a stream is up to the
plugin.

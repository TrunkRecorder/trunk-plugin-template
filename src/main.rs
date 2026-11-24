//! call-log — a Trunk Recorder Pro plugin that writes a line of JSON for
//! every recorded call.
//!
//! It's the template's example: replace it with your plugin. It shows the
//! parts every plugin has — settings (for the plugin, for each system, and
//! one a system can set again), a manifest, startup, handling an event,
//! reporting back, and shutting down.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use trunk_recorder_plugin::{ConcludedCall, Host, Manifest, Outcome, Plugin, Setup, topic};

/// The plugin's settings. The recorder draws a form from this (doc comments
/// become the labels and help text) and passes what the user entered.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct Config {
    /// Log file
    ///
    /// Where the lines go. A relative path is in the plugin's data folder.
    file: String,
    /// Log encrypted calls
    ///
    /// A system can set this again for itself (the manifest's `system_overrides`).
    include_encrypted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config { file: "calls.jsonl".into(), include_encrypted: false }
    }
}

/// Settings for each system (repeated under each in the form). Leaving a
/// system out needs no setting: Setup turns a plugin off for a system, and
/// the plugin is then told nothing of it.
#[derive(Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
struct SystemConfig {
    /// Label
    ///
    /// Written with its calls instead of the system's short name.
    label: String,
}

struct CallLog {
    host: Host,
    out: BufWriter<File>,
    /// Per system, by its id: the label, and whether its encrypted calls are logged.
    systems: HashMap<u32, (String, bool)>,
}

impl Plugin for CallLog {
    type Config = Config;
    type SystemConfig = SystemConfig;

    fn manifest() -> Manifest {
        Manifest {
            name: "Call log".into(),
            subscribe: vec![topic::CALL_CONCLUDED.into()],
            // The plugin settings a system may set again for itself.
            system_overrides: vec!["includeEncrypted".into()],
            // The id, version, description, … come from Cargo.toml.
            ..trunk_recorder_plugin::manifest!()
        }
    }

    fn start(host: Host, setup: Setup<Config, SystemConfig>) -> Result<Self, String> {
        // There's one log file: the plugin's own setting (`setup.config`).
        let mut path = PathBuf::from(&setup.config.file);
        if path.is_relative() {
            path = setup.data_dir.join(path);
        }
        // An Err here is shown to the user as a settings problem.
        let file = OpenOptions::new().create(true).append(true).open(&path).map_err(|e| format!("can't open {}: {e}", path.display()))?;
        let systems = setup
            .systems
            .iter()
            .map(|s| {
                let label = s.config.as_ref().map(|c| c.label.clone()).filter(|l| !l.is_empty()).unwrap_or_else(|| s.short_name.clone());
                // `s.settings` is the plugin's settings as this system has them (its overrides applied).
                (s.id, (label, s.settings.include_encrypted))
            })
            .collect();
        host.info(format!("logging calls to {}", path.display()));
        Ok(CallLog { host, out: BufWriter::new(file), systems })
    }

    fn call_concluded(&mut self, call: ConcludedCall) {
        // A system is known by its id (`system_id`), which stays when it's renamed; its short name is for showing.
        let Some((label, include_encrypted)) = self.systems.get(&call.call.system_id).cloned() else {
            self.host.call_result(&call.path, Outcome::Skipped, "unknown system", "");
            return;
        };
        if call.call.encrypted && !include_encrypted {
            self.host.call_result(&call.path, Outcome::Skipped, "encrypted", "");
            return;
        }
        let c = &call.call;
        let line = json!({
            "system": label,
            "talkgroup": c.talkgroup,
            "tag": c.talkgroup_tag,
            "start": c.start_time,
            // `audio_ms()`, not `call_length` (whole seconds).
            "ms": c.audio_ms(),
            "units": c.src_list.iter().map(|s| s.src).collect::<Vec<_>>(),
            "wav": call.files.wav,
        });
        let r = writeln!(self.out, "{line}").and_then(|_| self.out.flush());
        match r {
            Ok(()) => self.host.call_result(&call.path, Outcome::Ok, "", ""),
            Err(e) => self.host.call_result(&call.path, Outcome::Failed, e.to_string(), ""),
        }
    }

    fn shutdown(&mut self, _grace: std::time::Duration) {
        let _ = self.out.flush();
    }
}

fn main() {
    trunk_recorder_plugin::run::<CallLog>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use trunk_recorder_plugin::{HostMessage, testing};

    #[test]
    fn logs_a_call() {
        let dir = testing::temp_dir("call-log");
        let mut hello = testing::hello(&dir, json!({ "file": "out.jsonl" }));
        hello.systems[0].config = json!({ "label": "County" });
        let call = testing::call(&dir, "sys1", 101);
        let out = testing::run::<CallLog>([HostMessage::Hello(hello), HostMessage::CallConcluded(call.clone())]);
        assert!(out.ready());
        assert_eq!(out.results(), vec![(call.path, Outcome::Ok, String::new(), String::new())]);
        let line: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("data/out.jsonl")).unwrap()).unwrap();
        assert_eq!(line["system"], "County");
        assert_eq!(line["talkgroup"], 101);
        assert_eq!(line["ms"], 3000);
    }

    #[test]
    fn a_system_sets_its_own_setting_again() {
        let dir = testing::temp_dir("call-log");
        let mut hello = testing::hello(&dir, json!({ "includeEncrypted": false }));
        hello.systems[0].settings = json!({ "includeEncrypted": true });
        let mut call = testing::call(&dir, "sys1", 7);
        call.call.encrypted = true;
        let out = testing::run::<CallLog>([HostMessage::Hello(hello.clone()), HostMessage::CallConcluded(call.clone())]);
        assert_eq!(out.results()[0].1, Outcome::Ok, "the system's own setting");
        hello.systems[0].settings = Value::Null;
        let out = testing::run::<CallLog>([HostMessage::Hello(hello), HostMessage::CallConcluded(call)]);
        assert_eq!(out.results()[0].1, Outcome::Skipped, "the plugin's");
    }

    #[test]
    fn bad_settings_stop_it() {
        let dir = testing::temp_dir("call-log");
        let hello = testing::hello(&dir, json!({ "file": 42 }));
        let out = testing::run::<CallLog>([HostMessage::Hello(hello)]);
        assert!(!out.ready());
        assert_eq!(out.exit_code, trunk_recorder_plugin::EXIT_CONFIG);
    }
}

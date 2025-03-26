use crate::core::PluginError;

use std::{collections::BTreeMap, path::PathBuf};
use zellij_tile::{
    prelude::{LayoutInfo, PipeMessage, PipeSource},
    shim::get_plugin_ids,
};

// This structure mostly exists because `LayoutInfo` doesn't implement the `Default` trait.
#[derive(Debug)]
pub(super) struct PathFinderPluginConfig {
    /// Layout to load new sessions into. Defaults to the builtin `default` layout.
    pub(super) layout: LayoutInfo,

    /// Synthesized from the plugin startup configuration if it contains a `startup_message_name`
    /// key.
    pub(super) pipe_message: Option<PipeMessage>,

    /// Whether to automatically kill the session after switching.
    /// This is set to `true` in [PathFinderPluginConfig.load] if `pipe_message` is not `None`.
    pub(super) kill_after_switch: bool,
}

// Configuration.

/// See https://zellij.dev/documentation/plugin-aliases.html?highlight=caller#a-note-about-cwd.
const LAYOUT_OPTION: &'static str = "layout";

impl PathFinderPluginConfig {
    pub(super) fn load(&mut self, configuration: &BTreeMap<String, String>) {
        self.layout = parse_layout(&configuration.get(LAYOUT_OPTION));
        self.pipe_message = synthesize_pipe_message(configuration);
        self.kill_after_switch = self.pipe_message.is_some();
    }
}

/// The default builtin layout to use if the configuration does not specify one.
const DEFAULT_BUILTIN_LAYOUT: &'static str = "default";

/// Parses the configuration and determines the layout to use.
/// If the configuration does not specify a layout, the default builtin layout is used.
/// If the configuration specifies an unknown layout, the default builtin layout is used.
fn parse_layout(layout: &Option<&String>) -> LayoutInfo {
    let Some(layout) = layout else {
        return LayoutInfo::BuiltIn(DEFAULT_BUILTIN_LAYOUT.to_string());
    };
    let Some((scheme, name)) = layout.split_once(':') else {
        return LayoutInfo::BuiltIn(DEFAULT_BUILTIN_LAYOUT.to_string());
    };
    match scheme {
        "builtin" => LayoutInfo::BuiltIn(name.to_string()),
        "file" => LayoutInfo::File(name.to_string()),
        "stringified" => LayoutInfo::Stringified(name.to_string()),
        "url" => LayoutInfo::Url(name.to_string()),
        _ => LayoutInfo::BuiltIn(DEFAULT_BUILTIN_LAYOUT.to_string()),
    }
}

/// `name` is a reserved key, among others:
/// https://github.com/zellij-org/zellij/blob/afd4c644bc682df1bd9b06e575611aceb5e8c4a7/zellij-utils/src/input/layout.rs#L504-L516
const STARTUP_MESSAGE_NAME: &'static str = "startup_message_name";
const STARTUP_MESSAGE_PAYLOAD: &'static str = "startup_message_payload";

/// Synthesize a [PipeMessage] from the plugin config.
/// Returns `None` if `configuration` does not contain a `name` key.
fn synthesize_pipe_message(configuration: &BTreeMap<String, String>) -> Option<PipeMessage> {
    configuration
        .get(STARTUP_MESSAGE_NAME)
        .map(|name| PipeMessage {
            source: PipeSource::Plugin(get_plugin_ids().plugin_id),
            name: name.to_owned(),
            payload: configuration
                .get(STARTUP_MESSAGE_PAYLOAD)
                .map(|p| p.to_owned()),
            args: Default::default(),
            is_private: true,
        })
}

impl Default for PathFinderPluginConfig {
    fn default() -> Self {
        Self {
            layout: LayoutInfo::BuiltIn("default".to_string()),
            pipe_message: Default::default(),
            kill_after_switch: false,
        }
    }
}

// Pipe Messages.

/// The plugin configuration message name to pass to request using the builtin API to scan the
/// plugin's CWD and look for Git repositories.
///
/// ```kdl
/// MessagePlugin "pathfinder" {
///   cwd "/path/to/root/to/scan"
///   startup_message_name "scan_repository_root"
///   launch_new true
/// }
/// ```
///
/// Note that `launch_new` is required to guarantee that the plugin is restarted with the correct
/// CWD: Zellij plugins are jailed under their CWD, and cannot access the filesystem beyond it.
const PATHFINDER_COMMAND_SCAN_REPOSITORY_ROOT: &'static str = "scan_repository_root";

/// The plugin configuration message name to pass to request calling an external program to list
/// directories. This message expects an associated payload that is the absolute path to the cli
/// program to invoke.
///
/// ```kdl
/// MessagePlugin "pathfinder" {
///   startup_message_name "run_external_program"
///   startup_message_payload "/path/to/program/to/run"
/// }
/// ```
const PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM: &'static str = "run_external_program";

#[derive(Debug)]
pub(super) enum PathFinderPluginCommand {
    PluginCommandError(PluginError),
    ScanRepositoryRoot { max_depth: usize },
    RunExternalProgram { programs: Vec<PathBuf> },
}

impl From<PipeMessage> for PathFinderPluginCommand {
    fn from(message: PipeMessage) -> Self {
        match message.name.as_ref() {
            PATHFINDER_COMMAND_SCAN_REPOSITORY_ROOT => {
                parse_scan_repository_root_payload(message.name, message.payload)
            }
            PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM => {
                parse_run_external_program_payload(message.name, message.payload)
            }
            _ => PathFinderPluginCommand::PluginCommandError(PluginError::UnknownPipeMessageError(
                message.name,
            )),
        }
    }
}

fn parse_scan_repository_root_payload(
    name: String,
    payload: Option<String>,
) -> PathFinderPluginCommand {
    let Some(payload) = payload else {
        return PathFinderPluginCommand::ScanRepositoryRoot {
            max_depth: usize::MAX,
        };
    };

    let Ok(max_depth) = payload.parse::<usize>() else {
        return PathFinderPluginCommand::PluginCommandError(PluginError::ConfigurationError {
            reason: format!("{name}: invalid usize value: {payload}"),
        });
    };

    PathFinderPluginCommand::ScanRepositoryRoot { max_depth }
}

fn parse_run_external_program_payload(
    name: String,
    payload: Option<String>,
) -> PathFinderPluginCommand {
    let Some(payload) = payload else {
        return PathFinderPluginCommand::PluginCommandError(
            PluginError::MissingPipeMessagePayloadError(name),
        );
    };

    let programs = payload.split(":").map(PathBuf::from).collect();

    PathFinderPluginCommand::RunExternalProgram { programs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_run_external_program_payload_no_payload() {
        let message = PipeMessage {
            source: PipeSource::Plugin(0),
            name: PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM.to_string(),
            payload: None,
            args: Default::default(),
            is_private: true,
        };

        let command = PathFinderPluginCommand::from(message);

        assert!(matches!(
            command,
            PathFinderPluginCommand::PluginCommandError(
                PluginError::MissingPipeMessagePayloadError(name)
            ) if name == PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM
        ));
    }

    #[test]
    fn parse_run_external_program_payload_empty_payload() {
        let message = PipeMessage {
            source: PipeSource::Plugin(0),
            name: PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM.to_string(),
            payload: Some("".to_string()),
            args: Default::default(),
            is_private: true,
        };

        let command = PathFinderPluginCommand::from(message);

        assert!(matches!(
            command,
            PathFinderPluginCommand::RunExternalProgram { programs: _ }
        ));

        assert!(
            matches!(command, PathFinderPluginCommand::RunExternalProgram { programs } if programs.is_empty())
        );
    }

    #[test]
    fn parse_run_external_program_payload_single_program() {
        let message = PipeMessage {
            source: PipeSource::Plugin(0),
            name: PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM.to_string(),
            payload: Some("/path/to/program".to_string()),
            args: Default::default(),
            is_private: true,
        };

        let command = PathFinderPluginCommand::from(message);

        assert!(matches!(
            command,
            PathFinderPluginCommand::RunExternalProgram {
                programs
            } if programs == vec![PathBuf::from("/path/to/program")]
        ));
    }

    #[test]
    fn parse_run_external_program_payload_multiple_programs() {
        let message = PipeMessage {
            source: PipeSource::Plugin(0),
            name: PATHFINDER_COMMAND_RUN_EXTERNAL_PROGRAM.to_string(),
            payload: Some("/path/to/program1:/path/to/program2".to_string()),
            args: Default::default(),
            is_private: true,
        };

        let command = PathFinderPluginCommand::from(message);

        assert!(matches!(
            command,
            PathFinderPluginCommand::RunExternalProgram {
                programs
        } if programs == vec![
                    PathBuf::from("/path/to/program1"),
                    PathBuf::from("/path/to/program2")
                ]

        ));
    }
}

use serde::Serialize;
use serde_json::Value;

use crate::contracts::Action;

const MAX_OBS_SCENE_NAME_CHARS: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Allow,
    ApprovalRequired,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PolicyDecision {
    pub disposition: Disposition,
    pub code: &'static str,
    pub reason: &'static str,
}

pub fn evaluate(action: &Action) -> PolicyDecision {
    match action.kind() {
        "screen.capture" => screen_capture_policy(action),
        "filesystem.read" => PolicyDecision {
            disposition: Disposition::Allow,
            code: "read_only_capability",
            reason: "known read-only capability",
        },
        "shell.exec" => shell_policy(action),
        "input.click" | "input.type" | "app.launch" | "filesystem.write" => PolicyDecision {
            disposition: Disposition::ApprovalRequired,
            code: "effectful_capability",
            reason: "known effectful capability requires exact approval",
        },
        "obs.version.get"
        | "obs.scene.list"
        | "obs.scene.current"
        | "obs.record.status"
        | "obs.stream.status" => obs_policy(action, false),
        "obs.scene.set" | "obs.record.start" | "obs.record.stop" | "obs.stream.stop" => {
            obs_policy(action, true)
        }
        "obs.stream.start" => deny(
            "broadcast_start_disabled",
            "starting a public stream requires a stronger approval class that is not implemented",
        ),
        _ => PolicyDecision {
            disposition: Disposition::Deny,
            code: "unknown_capability",
            reason: "unknown action kinds fail closed",
        },
    }
}

fn screen_capture_policy(action: &Action) -> PolicyDecision {
    if !action.args().is_empty() {
        return deny(
            "screen_capture_arguments_forbidden",
            "Ubuntu portal screenshot capture accepts no model-controlled arguments",
        );
    }
    if !action.credential_handles().is_empty() {
        return deny(
            "screen_capture_credentials_forbidden",
            "desktop observation never receives credential handles",
        );
    }
    PolicyDecision {
        disposition: Disposition::Allow,
        code: "desktop_observation",
        reason: "one-shot Ubuntu Wayland screenshot capture is user-mediated by XDG Desktop Portal",
    }
}

fn obs_policy(action: &Action, effectful: bool) -> PolicyDecision {
    if action.credential_handles().len() > 1 {
        return deny(
            "obs_credential_arity",
            "OBS actions may bind at most one credential handle",
        );
    }

    let valid_port = action
        .args()
        .get("obs_port")
        .and_then(Value::as_u64)
        .is_some_and(|port| (1..=u64::from(u16::MAX)).contains(&port));
    if !valid_port {
        return deny(
            "obs_endpoint_unbound",
            "OBS actions must bind a non-zero loopback websocket port in args.obs_port",
        );
    }

    match action.kind() {
        "obs.scene.set" => {
            if action.args().len() != 2 {
                return deny(
                    "obs_invalid_arguments",
                    "obs.scene.set requires exactly obs_port and scene_name arguments",
                );
            }
            let Some(scene_name) = action.args().get("scene_name").and_then(Value::as_str) else {
                return deny(
                    "obs_invalid_arguments",
                    "obs.scene.set requires a string scene_name",
                );
            };
            if scene_name.trim().is_empty()
                || scene_name.chars().count() > MAX_OBS_SCENE_NAME_CHARS
                || scene_name.contains('\0')
            {
                return deny(
                    "obs_invalid_arguments",
                    "OBS scene names must be non-empty, bounded UTF-8 strings",
                );
            }
        }
        "obs.version.get"
        | "obs.scene.list"
        | "obs.scene.current"
        | "obs.record.status"
        | "obs.stream.status"
        | "obs.record.start"
        | "obs.record.stop"
        | "obs.stream.stop" => {
            if action.args().len() != 1 {
                return deny(
                    "obs_invalid_arguments",
                    "this OBS capability accepts only the bound obs_port argument",
                );
            }
        }
        _ => {
            return deny(
                "unknown_capability",
                "unknown OBS action kinds fail closed",
            );
        }
    }

    if effectful {
        PolicyDecision {
            disposition: Disposition::ApprovalRequired,
            code: "obs_effect",
            reason: "OBS state mutation requires exact human approval bound to the local endpoint",
        }
    } else {
        PolicyDecision {
            disposition: Disposition::Allow,
            code: "obs_read_only",
            reason: "known read-only OBS capability through the bound loopback broker",
        }
    }
}

fn shell_policy(action: &Action) -> PolicyDecision {
    let Some(Value::Array(argv)) = action.args().get("argv") else {
        return deny("invalid_argv", "shell.exec requires an argv array");
    };
    if argv.is_empty() || argv.iter().any(|item| !item.is_string()) {
        return deny("invalid_argv", "shell.exec argv must contain strings");
    }

    let strings = argv.iter().filter_map(Value::as_str).collect::<Vec<_>>();
    let program = strings[0].rsplit('/').next().unwrap_or(strings[0]);

    if matches!(program, "sudo" | "doas" | "su") {
        return deny("privilege_escalation", "privilege escalation commands are forbidden");
    }
    if contains_shell_command_string(&strings) {
        return deny("shell_escape", "shell interpreter command strings are forbidden");
    }
    if program.starts_with("mkfs") || matches!(program, "shutdown" | "reboot" | "poweroff" | "halt") {
        return deny("destructive_command", "destructive system commands are forbidden");
    }
    if program == "rm" {
        let recursive = strings.iter().skip(1).any(|arg| arg.contains('r') && arg.starts_with('-'));
        let force = strings.iter().skip(1).any(|arg| arg.contains('f') && arg.starts_with('-'));
        let root_target = strings.iter().skip(1).any(|arg| *arg == "/");
        if recursive && force && root_target {
            return deny("destructive_command", "recursive forced removal of / is forbidden");
        }
    }

    PolicyDecision {
        disposition: Disposition::ApprovalRequired,
        code: "shell_effect",
        reason: "structured shell execution requires exact approval",
    }
}

fn contains_shell_command_string(argv: &[&str]) -> bool {
    argv.iter().enumerate().any(|(index, token)| {
        is_shell_interpreter(token)
            && argv
                .iter()
                .skip(index + 1)
                .any(|argument| is_command_string_flag(argument))
    })
}

fn is_shell_interpreter(token: &str) -> bool {
    let name = token.rsplit('/').next().unwrap_or(token);
    matches!(
        name,
        "sh"
            | "bash"
            | "dash"
            | "zsh"
            | "fish"
            | "ksh"
            | "ksh93"
            | "mksh"
            | "pdksh"
            | "ash"
            | "yash"
            | "csh"
            | "tcsh"
            | "elvish"
            | "nu"
            | "nushell"
            | "pwsh"
            | "powershell"
    )
}

fn is_command_string_flag(argument: &str) -> bool {
    let lowered = argument.to_ascii_lowercase();
    if matches!(
        lowered.as_str(),
        "--command" | "-command" | "-encodedcommand"
    ) {
        return true;
    }

    lowered
        .strip_prefix('-')
        .is_some_and(|short| !short.starts_with('-') && short.contains('c'))
}

fn deny(code: &'static str, reason: &'static str) -> PolicyDecision {
    PolicyDecision {
        disposition: Disposition::Deny,
        code,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::ProposedAction;

    use super::*;

    fn action(json: &str) -> Action {
        let proposal: ProposedAction = match serde_json::from_str(json) {
            Ok(value) => value,
            Err(error) => panic!("fixture must parse: {error}"),
        };
        match proposal.normalize() {
            Ok(value) => value,
            Err(error) => panic!("fixture must normalize: {error}"),
        }
    }

    #[test]
    fn screen_capture_is_narrow_and_credential_free() {
        let capture = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture"}"#,
        );
        assert_eq!(evaluate(&capture).disposition, Disposition::Allow);

        let with_args = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture","args":{"target":"all"}}"#,
        );
        assert_eq!(evaluate(&with_args).disposition, Disposition::Deny);

        let with_credential = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture","credential_handles":["cred:openai.default"]}"#,
        );
        assert_eq!(evaluate(&with_credential).disposition, Disposition::Deny);
    }

    #[test]
    fn unknown_action_is_denied() {
        let action = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"teleport.cat"}"#);
        assert_eq!(evaluate(&action).disposition, Disposition::Deny);
    }

    #[test]
    fn safe_shell_requires_approval() {
        let action = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","hello"]}}"#);
        assert_eq!(evaluate(&action).disposition, Disposition::ApprovalRequired);
    }

    #[test]
    fn shell_escape_is_denied() {
        for argv in [
            r#"["bash","-c","echo nope"]"#,
            r#"["dash","-c","echo nope"]"#,
            r#"["bash","-lc","echo nope"]"#,
            r#"["/usr/bin/env","bash","-lc","echo nope"]"#,
            r#"["pwsh","-Command","Write-Output nope"]"#,
        ] {
            let json = format!(
                r#"{{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{{"argv":{argv}}}}}"#
            );
            let action = action(&json);
            assert_eq!(evaluate(&action).code, "shell_escape", "argv={argv}");
        }
    }

    #[test]
    fn obs_reads_are_allowed_but_mutations_require_approval() {
        let read = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.list","args":{"obs_port":4455}}"#,
        );
        let mutation = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.record.start","args":{"obs_port":4455}}"#,
        );
        assert_eq!(evaluate(&read).disposition, Disposition::Allow);
        assert_eq!(
            evaluate(&mutation).disposition,
            Disposition::ApprovalRequired
        );
    }

    #[test]
    fn obs_endpoint_must_be_bound_into_action() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.record.start"}"#,
        );
        assert_eq!(evaluate(&action).disposition, Disposition::Deny);
        assert_eq!(evaluate(&action).code, "obs_endpoint_unbound");
    }

    #[test]
    fn obs_stream_start_is_explicitly_denied() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.stream.start","args":{"obs_port":4455}}"#,
        );
        assert_eq!(evaluate(&action).disposition, Disposition::Deny);
        assert_eq!(evaluate(&action).code, "broadcast_start_disabled");
    }

    #[test]
    fn raw_obs_request_escape_is_denied() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.raw_request","args":{"obs_port":4455,"request_type":"StartStream"}}"#,
        );
        assert_eq!(evaluate(&action).disposition, Disposition::Deny);
        assert_eq!(evaluate(&action).code, "unknown_capability");
    }

    #[test]
    fn malformed_obs_scene_change_is_denied() {
        let action = action(
            r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"obs.scene.set","args":{"obs_port":4455,"scene_name":""}}"#,
        );
        assert_eq!(evaluate(&action).disposition, Disposition::Deny);
        assert_eq!(evaluate(&action).code, "obs_invalid_arguments");
    }
}

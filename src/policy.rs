use serde::Serialize;
use serde_json::Value;

use crate::contracts::Action;

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
        "screen.capture" | "filesystem.read" => PolicyDecision {
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
        _ => PolicyDecision {
            disposition: Disposition::Deny,
            code: "unknown_capability",
            reason: "unknown action kinds fail closed",
        },
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
    if matches!(program, "sh" | "bash" | "zsh" | "fish")
        && strings.iter().skip(1).any(|arg| *arg == "-c")
    {
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
        let action = action(r#"{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["bash","-c","echo nope"]}}"#);
        assert_eq!(evaluate(&action).code, "shell_escape");
    }
}

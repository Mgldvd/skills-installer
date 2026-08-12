/// Renders a human-readable, shell-quoted preview of a command for display
/// only (the "command preview" / dry-run feature). This string is **never**
/// executed — the real process always runs via `Command::new(program).args(args)`
/// with a `Vec<String>` argument array, never through a shell — so this
/// function's only job is to look correct to a human, not to be safe to
/// re-parse.
pub fn preview_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .map(|part| shell_quote(&part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:@%=".contains(c))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_plain_arguments_without_quoting() {
        assert_eq!(
            preview_command("skills", &["add".into(), "mattpocock/skills".into()]),
            "skills add mattpocock/skills"
        );
    }

    #[test]
    fn quotes_arguments_with_spaces() {
        assert_eq!(
            preview_command("skills", &["--agent".into(), "claude code".into()]),
            "skills --agent 'claude code'"
        );
    }

    #[test]
    fn escapes_embedded_single_quotes() {
        assert_eq!(preview_command("echo", &["it's".into()]), "echo 'it'\\''s'");
    }
}

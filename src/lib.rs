use std::io::{Write, Error};
use clap::{builder::StyledStr, Command};
use clap_complete::Generator;

/// Generate Clink completions
pub struct Clink;

impl Generator for Clink {
    fn file_name(&self, name: &str) -> String {
        format!("{name}.lua")
    }

    fn generate(&self, cmd: &Command, buf: &mut dyn Write) {
        self.try_generate(cmd, buf)
            .expect("failed to write completion file");
    }

    fn try_generate(&self, cmd: &Command, buf: &mut dyn Write) -> Result<(), Error> {
        let bin_name = cmd
            .get_bin_name()
            .expect("crate::generate should have set the bin_name");

        let inner = generate_inner(cmd).join("\n");

        write!(
            buf,
            r#"
clink.argmatcher("{bin_name}")
{inner}
"#
        )
    }
}

/// Generate completion method calls for a (sub)command.
/// The lines assume that they immediately follow a call to `clink.argmatcher`.
/// 
/// Clink completion definitions are nested, not flat, so lines are returned to make it easy to
/// indent them all.
fn generate_inner(cmd: &Command) -> Vec<String> {
    let mut lines = vec![];

    // `addarg` is positional. The next subcommand will always be first argument, so we need to
    // cover all subcommands in a single `addarg` call.
    let subcommands = cmd.get_subcommands().collect::<Vec<_>>();
    if !subcommands.is_empty() {
        lines.push(":addarg({".to_owned());

        for subcommand in &subcommands {
            lines.push(format!("    \"{}\"", subcommand.get_name()));

            // If there are no lines generated for the subcommand, it has no special parsing or
            // completion - e.g. no help, options or sub-subcommands.
            // In this case we can just leave it as a bare string.
            let subcommand_content = generate_inner(subcommand);
            if !subcommand_content.is_empty() {
                lines.push("        ..clink.argmatcher()".to_owned());
                for line in subcommand_content {
                    lines.push(format!("        {line}"));
                }
            }

            lines.last_mut().unwrap().push_str(", ");
        }

        lines.push("})".to_owned());

        // Add descriptions for commands that have them
        for subcommand in &subcommands {
            if let Some(about) = subcommand.get_about() {
                let about = escape_help(about);
                let visible_names = subcommand.get_name_and_visible_aliases();
                let command_names = lua_string_list(visible_names);

                lines.push(format!(":adddescriptions({{ {command_names}, description = \"{about}\" }})"));
            }
        }
    }

    // `addflags` isn't positional, so we can call it separately for every option.
    // This makes the generation code a little simpler.
    for opt in cmd.get_opts() {
        // Clink doesn't distinguish between long and short names
        let mut visible_names = vec![];
        if let Some(longs) = opt.get_long_and_visible_aliases() {
            for long in longs {
                visible_names.push(format!("--{long}"));
            }
        }
        if let Some(shorts) = opt.get_short_and_visible_aliases() {
            for short in shorts {
                visible_names.push(format!("-{short}"));
            }
        }

        let opt_names = lua_string_list(visible_names);
        let help = opt.get_help().map(escape_help).unwrap_or_default();

        lines.push(format!(":addflags({opt_names})"));
        lines.push(format!(":adddescriptions({{ {opt_names}, description = \"{help}\" }})"));
    }

    lines
}

/// Escape double quotes from a string.
fn escape_string(string: &str) -> String {
    string.replace('"', "\\\"")
}

/// Escape newlines from help text, while converting the styled string into a plain-text string.
fn escape_help(help: &StyledStr) -> String {
    escape_string(&help.to_string().replace('\n', " "))
}

/// Quote and comma-separate a sequence so it can be interpolated into a Lua argument list or table.
fn lua_string_list<S: AsRef<str>>(strs: Vec<S>) -> String {
    strs.iter()
        .map(|s| format!("\"{}\"", s.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}

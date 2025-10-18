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
    }

    // TODO: options
    // TODO: flags

    lines
}

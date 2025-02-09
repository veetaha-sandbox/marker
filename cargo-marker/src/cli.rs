use std::collections::HashMap;

use camino::Utf8Path;
use clap::{Args, Parser, Subcommand};

/// Marker's CLI interface
///
/// This binary should be invoked by Cargo with the new `marker` subcommand. If
/// you're reading this, consider manually adding `marker` as the first argument.
#[derive(Parser, Debug)]
struct MarkerApp {
    #[clap(subcommand)]
    subcommand: MarkerSubcommand,
}

#[derive(Parser, Debug)]
enum MarkerSubcommand {
    Marker(MarkerCli),
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(override_usage = "cargo marker [OPTIONS] [COMMAND] -- <CARGO ARGS>")]
pub struct MarkerCli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Used as the arguments to run Marker, when no command was specified
    #[clap(flatten)]
    pub check_args: CheckArgs,
}

impl MarkerCli {
    /// Prefer using this over the normal `parse` method, to split the arguments
    pub fn parse_args() -> Self {
        let MarkerSubcommand::Marker(cli) = MarkerApp::parse().subcommand;
        cli
    }
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Run Marker on the current package
    Check(CheckArgs),
    /// Setup the rustc driver for Marker
    Setup(SetupArgs),
    /// **UNSTABLE** Setup the specified lint crate for ui tests
    #[command(hide = true)]
    TestSetup(CheckArgs),
}

#[derive(Args, Debug)]
#[command(override_usage = "cargo marker check [OPTIONS] -- <CARGO ARGS>")]
pub struct CheckArgs {
    /// Specifies lint crates which should be used. (Lints in `Cargo.toml` will be ignored)
    #[arg(short, long)]
    pub lints: Vec<String>,
    /// Forwards the current `RUSTFLAGS` value during driver and lint crate compilation
    #[arg(long)]
    pub forward_rust_flags: bool,

    /// Arguments which will be forwarded to Cargo. See `cargo check --help`
    #[clap(last = true)]
    pub cargo_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct SetupArgs {
    /// Automatically installs the required toolchain using rustup
    #[arg(long)]
    pub auto_install_toolchain: bool,
    /// Forwards the current `RUSTFLAGS` value during driver and lint crate compilation
    #[arg(long)]
    pub forward_rust_flags: bool,
}

use crate::{
    config::{Config, ConfigFetchError, LintDependency},
    ExitStatus,
};

pub fn collect_lint_deps(args: &CheckArgs) -> Result<HashMap<String, LintDependency>, ExitStatus> {
    if args.lints.is_empty() {
        return Err(ExitStatus::NoLints);
    }

    let mut virtual_manifest = "[workspace.metadata.marker.lints]\n".to_string();
    for dep in &args.lints {
        virtual_manifest.push_str(dep);
        virtual_manifest.push('\n');
    }

    let path = Utf8Path::new(".");

    let Config { lints } = Config::try_from_str(&virtual_manifest, path).map_err(ConfigFetchError::emit_and_convert)?;
    Ok(lints)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        MarkerCli::command().debug_assert();
    }

    #[test]
    fn test_marker_cli() {
        let cli = MarkerCli::parse_from(&["cargo-marker", "check"]);
        assert!(matches!(cli.command, Some(CliCommand::Check(_))));

        let cli = MarkerCli::parse_from(&["cargo-marker"]);
        assert!(matches!(cli.command, None));
        assert!(cli.check_args.cargo_args.is_empty());

        let cli = MarkerCli::parse_from(&["cargo-marker", "--", "ducks", "penguins"]);
        assert!(matches!(cli.command, None));
        assert!(cli.check_args.cargo_args.len() == 2);
        assert!(cli.check_args.cargo_args[0] == "ducks");
        assert!(cli.check_args.cargo_args[1] == "penguins");

        let cli = MarkerCli::parse_from(&["cargo-marker", "check", "--", "ducks", "penguins"]);
        assert!(cli.check_args.cargo_args.is_empty());
        if let Some(CliCommand::Check(check_args)) = cli.command {
            assert!(check_args.cargo_args.len() == 2);
            assert!(check_args.cargo_args[0] == "ducks");
            assert!(check_args.cargo_args[1] == "penguins");
        } else {
            assert!(false, "the `check` subcommand was not detected");
        }
    }
}

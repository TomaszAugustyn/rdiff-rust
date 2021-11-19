use clap::Parser;
use std::path::PathBuf;

/// Compute signature-based file differences
#[derive(Parser)]
#[clap(
    name = "rdiff-rust",
    version = "0.1.0",
    author = "Tomasz Augustyn <t.augustyn@poczta.fm>"
)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

/// Enum representing possible subcommands
#[derive(Parser)]
pub enum SubCommand {
    #[clap(version = "0.1.0", author = "Tomasz Augustyn <t.augustyn@poczta.fm>")]
    Signature(Signature),
    Delta(Delta),
}

/// A subcommand for generating signature file for file before changes
#[derive(Parser)]
pub struct Signature {
    /// File before changes
    #[clap(name = "UNCHANGED_FILE", parse(from_os_str))]
    pub unchanged_file: PathBuf,
    /// Signature file
    #[clap(name = "SIGNATURE_FILE", parse(from_os_str))]
    pub signature_file: PathBuf,
}

/// A subcommand for creating delta using signature file and modified file
#[derive(Parser)]
pub struct Delta {
    /// Signature file
    #[clap(name = "SIGNATURE_FILE", parse(from_os_str))]
    pub signature_file: PathBuf,
    /// File after changes
    #[clap(name = "MODIFIED_FILE", parse(from_os_str))]
    pub modified_file: PathBuf,
    /// Delta file
    #[clap(name = "DELTA_FILE", parse(from_os_str))]
    pub delta_file: PathBuf,
}

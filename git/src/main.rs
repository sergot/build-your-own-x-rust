use anyhow::Context;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => {
            fs::create_dir(".git").context("create .git directory")?;
            fs::create_dir(".git/objects").context("create .git/objects directory")?;
            fs::create_dir(".git/refs").context("create .git/refs directory")?;
            fs::write(".git/HEAD", "ref: refs/heads/main").context("create .git/HEAD file")?;
            println!("Initialized git directory");
        }
    }
    Ok(())
}

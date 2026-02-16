use anyhow::Context;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

pub(crate) mod commands;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}

enum Kind {
    Blob,
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
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            commands::cat_file::invoke(pretty_print, &object_hash)?;
        }
        Command::HashObject { write, file } => {
            let hash = commands::hash_object::invoke(write, file)?;
            println!("{hash}");
        }
    }

    Ok(())
}

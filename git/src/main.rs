use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command as StdCommand;

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
            anyhow::ensure!(pretty_print, "only -p supported");
            let git_dir_cmd = StdCommand::new("git")
                .arg("rev-parse")
                .arg("--git-dir")
                .output()
                .expect("we should be in a git repo");
            let output = String::from_utf8_lossy(&git_dir_cmd.stdout);
            let git_dir = output.trim();
            let f = std::fs::File::open(format!(
                "{git_dir}/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("open a .git/objects file")?;

            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();

            z.read_until(0, &mut buf)
                .context("read header from object file")?;
            let header = CStr::from_bytes_with_nul(&buf).expect("there is only one nul");
            let header = header.to_str().context("header isn't valid utf8")?;
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!("wrong header: {header}");
            };
            let _kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("{kind} not supported yet"),
            };
            let size = size
                .parse::<u64>()
                .context(format!("git object file has an invalid size: {size}"))?;

            let mut z = z.take(size);
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            std::io::copy(&mut z, &mut stdout).context("write git object file to stdout")?;
        }
        Command::HashObject { write, file } => {
            anyhow::ensure!(write, "please use -w");
            let kind = "blob";
            let git_dir_cmd = StdCommand::new("git")
                .arg("rev-parse")
                .arg("--git-dir")
                .output()
                .expect("we should be in a git repo");
            let output = String::from_utf8_lossy(&git_dir_cmd.stdout);
            let git_dir = output.trim();

            let f = std::fs::File::open(format!("{}", file.to_string_lossy()))
                .context("open a source file")?;

            let mut br = BufReader::new(f);
            let mut buf = Vec::new();
            let size = br.read_to_end(&mut buf).context("read source file")?;
            let header = format!("{kind} {size}\0");
            let mut hasher = Sha1::new();
            hasher.update(header.as_bytes());
            hasher.update(&buf);
            let result = hasher.finalize();
            let hash = hex::encode(result);

            let tmp = "tmp";
            let tmpf = std::fs::File::create(tmp).context("create tmp file")?;
            let mut e = ZlibEncoder::new(tmpf, Compression::default());
            e.write_all(header.as_bytes())?;
            e.write_all(&buf)?;
            let _ = e.finish()?;

            std::fs::create_dir_all(format!("{git_dir}/objects/{}", &hash[..2]))
                .context("create git objects dir")?;
            std::fs::rename(
                tmp,
                format!("{git_dir}/objects/{}/{}", &hash[..2], &hash[2..]),
            )
            .context("move tmp file")?;

            println!("{hash}");
        }
    }

    Ok(())
}

use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command as StdCommand;

pub(crate) fn invoke(write: bool, file: PathBuf) -> anyhow::Result<String> {
    anyhow::ensure!(write, "please use -w");
    let kind = "blob";
    let git_dir_cmd = StdCommand::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .output()
        .expect("we should be in a git repo");
    let output = String::from_utf8_lossy(&git_dir_cmd.stdout);
    let git_dir = output.trim();

    let f =
        std::fs::File::open(format!("{}", file.to_string_lossy())).context("open a source file")?;

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

    Ok(hash)
}

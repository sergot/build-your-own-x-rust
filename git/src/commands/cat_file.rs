use crate::utils::find_git_dir;
use crate::Kind;
use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::BufReader;

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "only -p supported");
    let git_dir = find_git_dir().context("can't find git repository")?;
    let f = std::fs::File::open(
        git_dir
            .join("objects")
            .join(&object_hash[..2])
            .join(&object_hash[2..]),
    )
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

    Ok(())
}

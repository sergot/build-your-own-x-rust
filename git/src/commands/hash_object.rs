use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::path::Path;

use crate::utils::find_git_dir;

pub(crate) fn invoke(write: bool, file: &Path) -> anyhow::Result<String> {
    anyhow::ensure!(write, "please use -w");
    let kind = "blob";
    let git_dir = find_git_dir().context("can't find a git repository")?;

    let size = std::fs::metadata(file).context("stat source file")?.len();

    let mut source_file = std::fs::File::open(file).context("open a source file")?;

    let tmp = "tmp";
    let tmpf = std::fs::File::create(tmp).context("create tmp file")?;
    let z = ZlibEncoder::new(tmpf, Compression::default());
    let mut hashing_writer = HashingWriter {
        writer: z,
        hasher: Sha1::new(),
    };
    write!(hashing_writer, "{kind} {size}\0").context("writing header")?;
    std::io::copy(&mut source_file, &mut hashing_writer)
        .context("stream source file into tmp file")?;

    let _ = hashing_writer.writer.finish()?;
    let hash = hashing_writer.hasher.finalize();
    let hash = hex::encode(hash);

    let objects_path = git_dir.join("objects").join(&hash[..2]);

    std::fs::create_dir_all(&objects_path).context("create git objects dir")?;
    std::fs::rename(tmp, objects_path.join(&hash[2..])).context("move tmp file")?;

    Ok(hash)
}

struct HashingWriter<W> {
    writer: W,
    hasher: sha1::Sha1,
}

impl<W> Write for HashingWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.update(buf);
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

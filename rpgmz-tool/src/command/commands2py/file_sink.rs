use anyhow::ensure;
use anyhow::Context;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

/// A destination for bytes.
#[derive(Debug)]
pub enum FileSink {
    File {
        path: PathBuf,
        path_temp: PathBuf,
        file: BufWriter<File>,
    },
    Empty,
}

impl FileSink {
    /// Make a new [`FileSink`].
    pub fn new<P>(path: P, dry_run: bool, overwrite: bool) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        ensure!(
            overwrite
                || !path
                    .try_exists()
                    .context("failed to check if path exists")?,
            "output path \"{}\" already exists. Use the --overwrite flag to overwrite",
            path.display()
        );

        if dry_run {
            Ok(FileSink::new_empty())
        } else {
            FileSink::new_file(path)
        }
    }

    /// Create a new file variant.
    fn new_file<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_temp = nd_util::with_push_extension(path, "tmp");
        let file = File::create(&path_temp)
            .with_context(|| format!("failed to open \"{}\"", path_temp.display()))?;
        let file = BufWriter::new(file);

        Ok(Self::File {
            path: path.to_path_buf(),
            path_temp,
            file,
        })
    }

    /// Create a new empty variant.
    fn new_empty() -> Self {
        Self::Empty
    }

    /// Finish using this file sink, writing the result.
    pub fn finish(self) -> anyhow::Result<()> {
        match self {
            Self::File {
                path,
                path_temp,
                file,
            } => {
                let file = file.into_inner()?;
                file.sync_all()?;

                std::fs::rename(&path_temp, &path).with_context(|| {
                    format!(
                        "failed to rename file from \"{}\" to \"{}\"",
                        path_temp.display(),
                        path.display()
                    )
                })?;

                Ok(())
            }
            Self::Empty => Ok(()),
        }
    }
}

impl std::io::Write for FileSink {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::File { file, .. } => file.write(buffer),
            Self::Empty => Ok(buffer.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File { file, .. } => file.flush(),
            Self::Empty => Ok(()),
        }
    }
}

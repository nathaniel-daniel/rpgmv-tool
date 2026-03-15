use std::path::Path;

/// Try to get the metadata for a path.
pub fn try_metadata<P>(path: P) -> std::io::Result<Option<std::fs::Metadata>>
where
    P: AsRef<Path>,
{
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

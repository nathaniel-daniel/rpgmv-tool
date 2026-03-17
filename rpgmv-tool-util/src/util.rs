use anyhow::Context;
use std::path::Path;

pub fn try_exists<P>(path: P) -> anyhow::Result<bool>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    path.try_exists()
        .with_context(|| format!("failed to check if \"{}\" exists", path.display()))
}

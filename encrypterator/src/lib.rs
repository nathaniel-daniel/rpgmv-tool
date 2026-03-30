mod reader;

pub use self::reader::Reader;
use md5::Digest;
use md5::Md5;

const MAGIC_LEN: usize = 32;
const MAGIC: &[u8; MAGIC_LEN] = b"ART\0ENCRYPTER100FREE\0VERSION\0\0\0\0";
const TRANSFORMED_KEY_LEN: usize = 32;

/// Library error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid magic, expected {MAGIC:?} but got {actual:?}")]
    InvalidMagic { actual: [u8; MAGIC_LEN] },

    #[error("the provided buffer is too small")]
    BufferTooSmall,

    #[error("the key is missing")]
    MissingKey,

    #[error("io error")]
    Io(#[from] std::io::Error),
}

/// Prepare the encryption key.
///
/// This will always return a `Vec<u8>` with length 32.
fn transform_encryption_key(input: &str) -> [u8; TRANSFORMED_KEY_LEN] {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();

    let mut output = hash.to_vec();
    output.extend(hash.iter().rev());

    *output.as_array().expect("invalid key len")
}

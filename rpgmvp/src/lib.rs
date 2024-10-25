mod reader;
mod writer;

pub use self::reader::Reader;
pub use self::writer::Writer;

const MAGIC: [u8; 9] = *b"RPGMV\0\0\0\0";
const VERSION: [u8; 3] = [3, 1, 0];
const HEADER_PADDING: [u8; 4] = [0, 0, 0, 0];

const PNG_HEADER: &[u8] = b"\x89PNG\r\n\x1A\n\0\0\0\x0DIHDR";

/// The error type
#[derive(Debug, thiserror:: Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("invalid magic \"{magic:X?}\", expected \"{MAGIC:X?}\"")]
    InvalidMagic { magic: [u8; 9] },

    #[error("invalid version \"{version:?}\", expected \"{VERSION:?}\"")]
    InvalidVersion { version: [u8; 3] },

    #[error("invalid header padding \"{padding:?}\", expected \"{HEADER_PADDING:?}\"")]
    InvalidHeaderPadding { padding: [u8; 4] },

    #[error("the provided buffer is too small")]
    BufferTooSmall,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    /// File taken from https://github.com/kins-dev/rpgmaker_mv_decoder/blob/main/tests/assets/Actor1.rpgmvp
    const ENCRYPTED: &[u8] = include_bytes!("../test_data/Actor1.rpgmvp");

    #[test]
    fn reader() {
        let mut reader = Reader::new(ENCRYPTED);
        let mut writer =
            File::create("test-temp/reader-decrypted.png").expect("failed to open output");
        std::io::copy(&mut reader, &mut writer).expect("failed to copy");
    }

    #[test]
    fn reader_writer() {
        let mut reader = Reader::new(ENCRYPTED);
        let mut decrypted = Vec::new();
        std::io::copy(&mut reader, &mut decrypted).expect("failed to copy");
        let key = reader.extract_key().expect("failed to extract key");

        let mut encrypted: Vec<u8> = Vec::new();
        let mut writer = Writer::new(&mut encrypted, key);
        writer.write_header().expect("failed to write header");
        writer.write_all(&decrypted).expect("failed to write body");

        assert!(ENCRYPTED == encrypted);
    }
}

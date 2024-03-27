use std::io::BufRead;
use std::io::Read;

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

    #[error("the reader cannot extract the key as it has already moved past it")]
    PastKey,
}

/// The reader state
enum ReaderState {
    /// Reads header next
    Header,
    /// Reads the start of the body next, but no key has been determined
    BodyInitialNoKey,
    /// Reads the start of the body next, but a key has been determined
    BodyInitial { key: [u8; 16], offset: usize },
    /// Reads the body next, but is past the beginning
    Body,
}

/// A reader for an encrypted PNG file
pub struct Reader<R> {
    reader: R,
    state: ReaderState,
}

impl<R> Reader<R> {
    /// Create a new reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            state: ReaderState::Header,
        }
    }
}

impl<R> Reader<R>
where
    R: BufRead,
{
    /// Read and validate the header.
    pub fn read_header(&mut self) -> Result<(), Error> {
        let buffer = self.reader.fill_buf()?;
        let header = buffer.get(..16).ok_or(Error::BufferTooSmall)?;

        let magic: [u8; 9] = header[..9].try_into().unwrap();
        let version: [u8; 3] = header[9..12].try_into().unwrap();
        let padding: [u8; 4] = header[12..].try_into().unwrap();

        if magic != MAGIC {
            return Err(Error::InvalidMagic { magic });
        }

        if version != VERSION {
            return Err(Error::InvalidVersion { version });
        }

        if padding != HEADER_PADDING {
            return Err(Error::InvalidHeaderPadding { padding });
        }

        let header_len = header.len();
        self.reader.consume(header_len);
        self.state = ReaderState::BodyInitialNoKey;

        Ok(())
    }

    /// Determine the encryption key.
    pub fn extract_key(&mut self) -> Result<[u8; 16], Error> {
        loop {
            match self.state {
                ReaderState::Header => {
                    self.read_header()?;
                }
                ReaderState::BodyInitialNoKey | ReaderState::BodyInitial { .. } => break,
                ReaderState::Body { .. } => {
                    return Err(Error::PastKey);
                }
            }
        }

        let buffer = self.reader.fill_buf()?;
        let png_header = buffer.get(..16).ok_or(Error::BufferTooSmall)?;

        let mut key = [0; 16];
        for ((expected_png_byte, actual_png_byte), key_byte) in PNG_HEADER
            .iter()
            .copied()
            .zip(png_header.iter().copied())
            .zip(key.iter_mut())
        {
            *key_byte = actual_png_byte ^ expected_png_byte;
        }

        self.state = ReaderState::BodyInitial { key, offset: 0 };

        Ok(key)
    }
}

impl<R> Read for Reader<R>
where
    R: BufRead,
{
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match &mut self.state {
                ReaderState::Header => {
                    self.read_header()
                        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
                }
                ReaderState::BodyInitialNoKey => {
                    self.extract_key()
                        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
                }
                ReaderState::BodyInitial { key, offset } => {
                    let n = self.reader.read(buffer)?;

                    for (key_byte, out_byte) in
                        key[*offset..].iter().copied().zip(buffer[..n].iter_mut())
                    {
                        *out_byte ^= key_byte;
                        *offset += 1;
                    }

                    if *offset == 16 {
                        self.state = ReaderState::Body;
                    }

                    return Ok(n);
                }
                ReaderState::Body { .. } => {
                    return self.reader.read(buffer);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    /// File taken from https://github.com/kins-dev/rpgmaker_mv_decoder/blob/main/tests/assets/Actor1.rpgmvp
    const ENCRYPTED: &[u8] = include_bytes!("../test_data/Actor1.rpgmvp");

    #[test]
    fn reader() {
        let mut reader = Reader::new(ENCRYPTED);
        let mut writer =
            File::create("test-temp/reader-decrypted.png").expect("failed to open output");
        std::io::copy(&mut reader, &mut writer).expect("failed to copy");
    }
}

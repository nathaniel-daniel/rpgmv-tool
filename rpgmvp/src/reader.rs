use crate::Error;
use crate::HEADER_PADDING;
use crate::MAGIC;
use crate::PNG_HEADER;
use crate::VERSION;
use std::io::BufRead;
use std::io::Read;

/// The reader state
enum ReaderState {
    /// Reads header next
    Header,
    /// Reads the start of the body next, but no key has been determined
    BodyInitialNoKey,
    /// Reads the start of the body next, but a key has been determined
    BodyInitial { key: [u8; 16], offset: usize },
    /// Reads the body next, but is past the beginning
    Body { key: [u8; 16] },
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
        match self.state {
            ReaderState::Header => {}
            _ => return Ok(()),
        }

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
                ReaderState::BodyInitialNoKey => break,
                ReaderState::BodyInitial { key, .. } | ReaderState::Body { key } => {
                    return Ok(key);
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
                        self.state = ReaderState::Body { key: *key };
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

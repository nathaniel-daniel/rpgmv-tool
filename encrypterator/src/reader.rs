use crate::Error;
use crate::MAGIC;
use crate::MAGIC_LEN;
use crate::TRANSFORMED_KEY_LEN;
use crate::transform_encryption_key;

const PNG_HEADER: &[u8] = b"\x89PNG\r\n\x1A\n\0\0\0\x0DIHDR";

#[derive(Debug)]
enum State {
    /// Need to read the magic.
    Magic,
    /// The magic has been read, but they key is missing.
    BodyStartNoKey,
    /// The magic has been read, and the encrypted part of the body must now be read.
    BodyStart {
        key: [u8; TRANSFORMED_KEY_LEN],
        offset: usize,
    },
    /// The encrypted magic has been read, and now the unencrypted rest of the file must be read.
    Body,
}

/// A struct to decrypt from a reader.
#[derive(Debug)]
pub struct Reader<R> {
    reader: R,
    state: State,
}

impl<R> Reader<R> {
    /// Make a new Reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            state: State::Magic,
        }
    }
}

impl<R> Reader<R>
where
    R: std::io::BufRead,
{
    /// Read and verify the magic, if it hasn't been done yet.
    pub fn read_magic(&mut self) -> Result<(), Error> {
        match &self.state {
            State::Magic => {}
            State::BodyStartNoKey | State::BodyStart { .. } | State::Body => return Ok(()),
        }

        let mut buffer = [0; MAGIC_LEN];
        self.reader.read_exact(&mut buffer)?;

        if buffer != *MAGIC {
            return Err(Error::InvalidMagic { actual: buffer });
        }
        self.state = State::BodyStartNoKey;

        Ok(())
    }

    /// Determine the encryption key, in its transformed state.
    ///
    /// If the key has already been determined by any means, this is a nop.
    ///
    /// This currently only works for encrypted pngs,
    /// since they provide the minimum 16 bytes we need to recover the key.
    ///
    /// However, not all assets as pngs.
    /// Some assets are oggs, which do not provide the minimum 16 bytes.
    pub fn guess_key(&mut self) -> Result<(), Error> {
        loop {
            match self.state {
                State::Magic => {
                    self.read_magic()?;
                }
                State::BodyStartNoKey => break,
                State::BodyStart { .. } | State::Body => {
                    return Ok(());
                }
            }
        }

        let buffer = self.reader.fill_buf()?;
        let png_header = buffer
            .get(..TRANSFORMED_KEY_LEN)
            .ok_or(Error::BufferTooSmall)?;

        let mut half_key_1 = [0; TRANSFORMED_KEY_LEN / 2];
        for ((expected_png_byte, actual_png_byte), key_byte) in PNG_HEADER
            .iter()
            .copied()
            .zip(png_header.iter().copied())
            .zip(half_key_1.iter_mut())
        {
            *key_byte = actual_png_byte ^ expected_png_byte;
        }

        // To generate the full transformed key from the first 16 bytes of the transformed key,
        // simply reverse the key and add it to a buffer.
        let half_key_2: Vec<u8> = half_key_1.iter().copied().rev().collect();
        let mut key = [0; TRANSFORMED_KEY_LEN];
        key[..TRANSFORMED_KEY_LEN / 2].copy_from_slice(&half_key_1);
        key[TRANSFORMED_KEY_LEN / 2..].copy_from_slice(&half_key_2);

        self.state = State::BodyStart { key, offset: 0 };

        Ok(())
    }

    /// Set the key manually.
    ///
    /// This will transform the key before use.
    /// This prevents the Reader from automatically guessing the key.
    pub fn set_key(&mut self, key: &str) -> Result<(), Error> {
        loop {
            match self.state {
                State::Magic => {
                    self.read_magic()?;
                }
                State::BodyStartNoKey => break,
                State::BodyStart { .. } | State::Body => {
                    return Ok(());
                }
            }
        }

        let key = transform_encryption_key(key);

        self.state = State::BodyStart { key, offset: 0 };

        Ok(())
    }
}

impl<R> std::io::Read for Reader<R>
where
    R: std::io::BufRead,
{
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match &mut self.state {
                State::Magic => {
                    self.read_magic().map_err(std::io::Error::other)?;
                }
                State::BodyStartNoKey => {
                    return Err(std::io::Error::other(Error::MissingKey));
                }
                State::BodyStart { key, offset } => {
                    let buffer_len = std::cmp::min(buffer.len(), TRANSFORMED_KEY_LEN - *offset);
                    let n = self.reader.read(&mut buffer[..buffer_len])?;

                    for (key_byte, out_byte) in
                        key[*offset..].iter().copied().zip(buffer[..n].iter_mut())
                    {
                        *out_byte ^= key_byte;
                        *offset += 1;
                    }

                    if *offset == TRANSFORMED_KEY_LEN {
                        self.state = State::Body;
                    }

                    return Ok(n);
                }
                State::Body => {
                    return self.reader.read(buffer);
                }
            }
        }
    }
}

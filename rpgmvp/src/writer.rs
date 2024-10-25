use crate::Error;
use crate::HEADER_PADDING;
use crate::MAGIC;
use crate::VERSION;
use std::io::Write;

/// The reader state
#[derive(Debug, PartialEq)]
enum WriterState {
    /// Writes header next
    Header,

    /// Reads the start of the body next
    BodyInitial { offset: usize },

    /// Writes the body next
    Body,
}

impl WriterState {
    /// Check if this is the header state.
    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header)
    }
}

/// An ecrypted png writer.
#[derive(Debug)]
pub struct Writer<W> {
    writer: W,
    state: WriterState,
    key: [u8; 16],
}

impl<W> Writer<W> {
    /// Make a new writer.
    pub fn new(writer: W, key: [u8; 16]) -> Self {
        Self {
            writer,
            state: WriterState::Header,
            key,
        }
    }
}

impl<W> Writer<W>
where
    W: Write,
{
    /// Write the header.
    pub fn write_header(&mut self) -> Result<(), Error> {
        if !self.state.is_header() {
            return Ok(());
        }

        self.writer.write_all(&MAGIC)?;
        self.writer.write_all(&VERSION)?;
        self.writer.write_all(&HEADER_PADDING)?;
        self.state = WriterState::BodyInitial { offset: 0 };

        Ok(())
    }
}

impl<W> Write for Writer<W>
where
    W: Write,
{
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        loop {
            match &mut self.state {
                WriterState::Header => {
                    self.write_header()
                        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
                }
                WriterState::BodyInitial { offset } => {
                    let mut written = 0;
                    for (input_byte, key_byte) in buffer
                        .iter()
                        .copied()
                        .zip(self.key[*offset..].iter().copied())
                    {
                        let out_byte = input_byte ^ key_byte;
                        self.writer.write_all(&[out_byte])?;
                        written += 1;
                    }
                    *offset += written;

                    if *offset == 16 {
                        self.state = WriterState::Body;
                    }

                    return Ok(written);
                }
                WriterState::Body => {
                    return self.writer.write(buffer);
                }
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

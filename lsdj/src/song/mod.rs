//! Unparsed LSDJ song memory

pub mod instrument;
mod offsets;
pub(crate) mod wave;

use instrument::Instruments;
use std::io::{self, Read, Write};
use thiserror::Error;

/// A contiguous block of memory that represents unparsed song data
///
/// Future versions of this create might parse [`SongMemory`] into different formatted versions
/// of songs, but for now this suffices to import and export songs from [`SRam`](crate::sram).
pub struct SongMemory {
    /// The bytes that make up the song
    bytes: Vec<u8>,
}

impl SongMemory {
    /// The number of bytes taken up by a single LSDJ song
    pub const LEN: usize = 0x8000;

    /// Construct a new, empty song, ready for use
    ///
    /// This sets all the necessary verification bytes that LSDJ uses to check for memory corruption.
    pub fn new() -> Self {
        Self {
            bytes: include_bytes!("92L_empty.raw").to_vec(),
        }
    }

    /// Deserialize [`SongMemory`] from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
        if bytes.len() != Self::LEN {
            return Err(FromBytesError::IncorrectSize);
        }

        let check = |offset| bytes[offset] == 0x72 && bytes[offset + 1] == 0x62;

        if check(0x1E78) || check(0x3E80) || check(0x7FF0) {
            Ok(Self {
                bytes: bytes.to_vec(),
            })
        } else {
            Err(FromBytesError::InitializationCheckIncorrect)
        }
    }

    /// Deserialize [`SongMemory`] from an arbitrary I/O reader
    pub fn from_reader<R>(mut reader: R) -> Result<Self, FromReaderError>
    where
        R: Read,
    {
        let mut bytes = [0; Self::LEN];
        reader.read_exact(bytes.as_mut_slice())?;

        let song = Self::from_bytes(&bytes)?;

        Ok(song)
    }

    /// Serialize [`SongMemory`] to an arbitrary I/O writer
    pub fn to_writer<W>(&self, mut writer: W) -> Result<(), io::Error>
    where
        W: Write,
    {
        writer.write_all(&self.bytes)
    }

    /// The version of the format the song is encoded in
    pub fn format_version(&self) -> u8 {
        self.bytes[self::offsets::FORMAT_VERSION_OFFSET]
    }

    /// Access the bytes that make up the song
    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Access the bytes that make up the song
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.bytes.as_mut_slice()
    }

    pub fn instruments<'a>(&'a self) -> Instruments<'a> {
        Instruments::new(self)
    }
}

impl Default for SongMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that might be returned from [`SongMemory::from_bytes()`]
#[derive(Debug, Error)]
pub enum FromBytesError {
    /// The passed in number of bytes isn't correct
    #[error("The slice isn't of the correct size")]
    IncorrectSize,

    /// All correctly initialized song memory has certain bytes set for
    /// verification against memory corruption.
    ///
    /// This error is returned when that those bytes are faulty during a read.
    #[error("The initialization check failed")]
    InitializationCheckIncorrect,
}

/// Errors that might be returned from [`SongMemory::from_reader()`]
#[derive(Debug, Error)]
pub enum FromReaderError {
    /// Reading the bytes failed
    #[error("Something failed with I/O")]
    Read(#[from] io::Error),

    /// Deserialization from the read bytes failed
    #[error("Deserialiazation from the read bytes failed")]
    FromBytes(#[from] FromBytesError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_92l() {
        use std::io::Cursor;

        let song = {
            let bytes = Cursor::new(include_bytes!("../../test/92L_empty.sav"));
            SongMemory::from_reader(bytes).expect("could not parse song")
        };

        assert_eq!(song.format_version(), 0x16);
    }
}

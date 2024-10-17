use crate::song::{instrument::get_instrument_bits, SongMemory};

#[derive(Debug)]
pub struct Noise {
    length: NoiseLength,
    shape: u8,
    stability: NoiseStability,
    envelope: (u8, u8, u8),
}

#[derive(Debug)]
pub enum NoiseLength {
    Infinite,
    Finite(u8),
}

#[derive(Debug)]
pub enum NoiseStability {
    Free = 0,
    Stable,
}

#[derive(thiserror::Error, Debug)]
pub enum FromBytesError {
    #[error("Unknown noise stability {0}")]
    UnknownNoiseStability(u8),
}

impl NoiseStability {
    pub fn from_bytes(song: &SongMemory, instrument: usize) -> Result<Self, FromBytesError> {
        let raw = get_instrument_bits(song, instrument, 2, 0, 1);

        let stability = match raw {
            0 => NoiseStability::Free,
            1 => NoiseStability::Stable,
            _ => return Err(FromBytesError::UnknownNoiseStability(raw)),
        };

        Ok(stability)
    }
}

impl Noise {
    pub fn from_bytes(song: &SongMemory, instrument: usize) -> Result<Self, FromBytesError> {
        let length = if get_instrument_bits(song, instrument, 3, 6, 1) == 0 {
            NoiseLength::Infinite
        } else {
            NoiseLength::Finite((!get_instrument_bits(song, instrument, 3, 0, 5)) & 0x3F)
        };

        let shape = get_instrument_bits(song, instrument, 4, 0, 8);
        let stability = NoiseStability::from_bytes(song, instrument)?;
        let envelope = (
            get_instrument_bits(song, instrument, 1, 0, 8),
            get_instrument_bits(song, instrument, 9, 0, 8),
            get_instrument_bits(song, instrument, 10, 0, 8),
        );

        Ok(Self {
            envelope,
            length,
            shape,
            stability,
        })
    }
}

use crate::song::SongMemory;

use super::get_instrument_bits;

#[derive(Debug)]
pub struct Pulse {
    width: PulseWidth,
    length: PulseLength,
    sweep: u8,
    tune: u8,
    finetune: u8,
    envelope: (u8, u8, u8),
}

#[derive(Debug)]
pub enum PulseLength {
    Infinite,
    Finite(u8),
}

#[derive(Debug)]
pub enum PulseWidth {
    Width125 = 0,
    Width25,
    Width50,
    Width75,
}

#[derive(thiserror::Error, Debug)]
pub enum FromBytesError {
    #[error("Unknown pulse width {0}")]
    UnknownPulseWidth(u8),
}

impl PulseWidth {
    pub fn from_bytes(song: &SongMemory, instrument: usize) -> Result<Self, FromBytesError> {
        let raw = get_instrument_bits(song, instrument, 7, 6, 2);
        let width = match raw {
            0 => PulseWidth::Width125,
            1 => PulseWidth::Width25,
            2 => PulseWidth::Width50,
            3 => PulseWidth::Width75,
            _ => return Err(FromBytesError::UnknownPulseWidth(raw)),
        };

        Ok(width)
    }
}

impl Pulse {
    pub fn from_bytes(song: &SongMemory, instrument: usize) -> Result<Self, FromBytesError> {
        let width = PulseWidth::from_bytes(song, instrument)?;
        let sweep = get_instrument_bits(song, instrument, 4, 0, 8);
        let tune = get_instrument_bits(song, instrument, 2, 0, 8);
        let finetune = get_instrument_bits(song, instrument, 7, 2, 4);
        let length = if get_instrument_bits(song, instrument, 3, 6, 1) == 0 {
            PulseLength::Infinite
        } else {
            PulseLength::Finite((!get_instrument_bits(song, instrument, 3, 0, 5)) & 0x3F)
        };

        let envelope = (
            get_instrument_bits(song, instrument, 1, 0, 8),
            get_instrument_bits(song, instrument, 9, 0, 8),
            get_instrument_bits(song, instrument, 10, 0, 8),
        );

        Ok(Self {
            width,
            length,
            sweep,
            tune,
            finetune,
            envelope,
        })
    }
}

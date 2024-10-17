use core::fmt;

use crate::name::{self, Name};

use super::SongMemory;

mod noise;
mod pulse;
mod wave;

/// The default string of bytes for a new instrument
pub const DEFAULT_INSTRUMENT: [u8; 16] = [
    0xA8, 0x0, 0x0, 0xFF, 0x0, 0x0, 0x3, 0x0, 0x0, 0xD0, 0x0, 0x0, 0x0, 0xF3, 0x0, 0x0,
];

/// The beginning address within a SongMemory of the first instrument name
const INSTRUMENT_NAMES_OFFSET: usize = 0x1E7A;

/// The beginning address within a SongMemory of the first instrument parameter set
const INSTRUMENT_PARAMS_OFFSET: usize = 0x3080;

/// The amount of instruments in a song
const INSTRUMENT_COUNT: usize = 0x40;

/// The amount of bytes an instrument takes
const INSTRUMENT_BYTE_COUNT: usize = 16;

/// The amount of bytes an instrument name takes
const INSTRUMENT_NAME_LENGTH: usize = 5;

const INSTRUMENT_ALLOCATION_TABLE_OFFSET: usize = 0x2040;

/// The value of an infinite pulse length
const INSTRUMENT_PULSE_LENGTH_INFINITE: u8 = 0x40;

/// The value of a kit length set to AUTO
const INSTRUMENT_KIT_LENGTH_AUTO: u8 = 0x0;

/// The value of an infinite noise length
const INSTRUMENT_NOISE_LENGTH_INFINITE: u8 = 0x40;

#[derive(thiserror::Error, Debug)]
pub enum FromBytesError {
    #[error("Instrument {index} has invalid name {bytes:?}")]
    Name {
        #[source]
        error: name::FromBytesError,
        index: usize,
        bytes: Vec<u8>,
    },

    #[error("Instrument {index} has invalid kind {kind}")]
    InstrumentKind { index: usize, kind: u8 },

    #[error("Instrument {index} is a Pulse type, but the inner pulse data was invalid.")]
    Pulse {
        #[source]
        error: pulse::FromBytesError,
        index: usize,
    },

    #[error("Instrument {index} is a Wave type, but the inner wave data was invalid.")]
    Wave {
        #[source]
        error: wave::FromBytesError,
        index: usize,
    },

    #[error("Instrument {index} is a Noise type, but the inner noise data was invalid.")]
    Noise {
        #[source]
        error: noise::FromBytesError,
        index: usize,
    },
}

pub struct Instruments<'a> {
    index: usize,
    song: &'a SongMemory,
}

#[derive(Debug)]
pub struct Instrument {
    name: Name<INSTRUMENT_NAME_LENGTH>,
    kind: Kind,
    index: usize,
}

#[derive(Debug)]
pub enum Kind {
    Pulse(pulse::Pulse),
    Wave(wave::Wave),
    Kit,
    Noise(noise::Noise),
}

// impl fmt::Debug for Kind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Kind::Pulse(pulse) => write!(f, "{pulse:?}"),
//             Kind::Wave(wave) => write!(f, "{wave:?}"),
//             Kind::Kit => write!(f, "Kit"),
//             Kind::Noise(noise) => write!(f, "{noise:?}"),
//         }
//     }
// }

// impl fmt::Debug for Instrument {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Instrument")
//             .field("name", &self.name.as_str())
//             .field("kind", &self.kind)
//             .field("index", &self.index)
//             .finish()
//     }
// }

impl Instrument {
    fn from_bytes(song: &SongMemory, index: usize) -> Result<Instrument, FromBytesError> {
        let name_offset = index * INSTRUMENT_NAME_LENGTH + INSTRUMENT_NAMES_OFFSET;
        let name_slice = &song.as_slice()[name_offset..(name_offset + INSTRUMENT_NAME_LENGTH)];
        let name = Name::from_bytes(name_slice).map_err(|error| FromBytesError::Name {
            error,
            index,
            bytes: name_slice.to_vec(),
        })?;
        let kind = get_instrument_kind(song, index)?;

        let instrument = Instrument { name, kind, index };

        Ok(instrument)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

impl<'a> Instruments<'a> {
    pub(crate) fn new(song: &'a SongMemory) -> Self {
        Self { song, index: 0 }
    }
}

impl<'a> Iterator for Instruments<'a> {
    type Item = Result<Instrument, FromBytesError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < INSTRUMENT_COUNT {
            if !is_instrument_allocated(self.song, self.index) {
                self.index += 1;
                continue;
            }

            let maybe_instrument = Instrument::from_bytes(self.song, self.index);

            self.index += 1;

            return Some(maybe_instrument);
        }

        None
    }
}

fn get_instrument_kind(song: &SongMemory, index: usize) -> Result<Kind, FromBytesError> {
    let data = get_instrument_bits(song, index, 0, 0, 8);

    let kind = match data {
        0 => Kind::Pulse(
            pulse::Pulse::from_bytes(song, index)
                .map_err(|e| FromBytesError::Pulse { error: e, index })?,
        ),
        1 => Kind::Wave(
            wave::Wave::from_bytes(song, index)
                .map_err(|e| FromBytesError::Wave { error: e, index })?,
        ),
        2 => Kind::Kit,
        3 => Kind::Noise(
            noise::Noise::from_bytes(song, index)
                .map_err(|e| FromBytesError::Noise { error: e, index })?,
        ),
        kind => return Err(FromBytesError::InstrumentKind { index, kind }),
    };

    Ok(kind)
}

fn get_instrument_bits(
    song: &SongMemory,
    instrument_idx: usize,
    byte_idx: usize,
    position: u8,
    count: u8,
) -> u8 {
    let index: usize = instrument_idx * INSTRUMENT_BYTE_COUNT + byte_idx;
    assert!(index < 1024);
    let offset: usize = INSTRUMENT_PARAMS_OFFSET + index;

    let byte = song.as_slice()[offset];
    get_bits(byte, position, count) >> position
}

fn get_bits(byte: u8, position: u8, count: u8) -> u8 {
    byte & (create_mask(count) << position)
}

fn create_mask(count: u8) -> u8 {
    ((1u16 << count) - 1) as u8
}

pub fn is_instrument_allocated(song: &SongMemory, instrument_idx: usize) -> bool {
    let index = INSTRUMENT_ALLOCATION_TABLE_OFFSET + instrument_idx;
    assert!(index <= INSTRUMENT_ALLOCATION_TABLE_OFFSET + 64);

    song.as_slice()[index] != 0
}

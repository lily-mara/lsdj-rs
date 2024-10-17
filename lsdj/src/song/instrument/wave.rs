use crate::song::{instrument::get_instrument_bits, SongMemory};

#[derive(Debug)]
pub struct Wave {
    volume: u8,
    synth: u8,
    wave: u8,
    play_mode: WavePlayMode,
    length: u8,
    loop_pos: u8,
    repeat: u8,
    speed: u8,
}

#[derive(Debug)]
pub enum WavePlayMode {
    Once,
    Loop,
    PingPong,
    Manual,
}

#[derive(thiserror::Error, Debug)]
pub enum FromBytesError {
    #[error("Unknown wave play mode {0}")]
    UnknownWavePlayMode(u8),
}

impl WavePlayMode {
    pub fn from_bytes(song: &SongMemory, index: usize) -> Result<Self, FromBytesError> {
        let raw = if song.format_version() >= 10 {
            get_instrument_bits(song, index, 9, 0, 2).wrapping_sub(1) & 0x3
        } else {
            get_instrument_bits(song, index, 9, 0, 2)
        };

        let play_mode = match raw {
            0 => WavePlayMode::Once,
            1 => WavePlayMode::Loop,
            2 => WavePlayMode::PingPong,
            3 => WavePlayMode::Manual,
            _ => return Err(FromBytesError::UnknownWavePlayMode(raw)),
        };

        Ok(play_mode)
    }
}

impl Wave {
    pub fn from_bytes(song: &SongMemory, index: usize) -> Result<Self, FromBytesError> {
        let format_version = song.format_version();

        let volume = get_instrument_bits(song, index, 1, 0, 8);
        let synth = if format_version >= 16 {
            get_instrument_bits(song, index, 3, 4, 4)
        } else {
            get_instrument_bits(song, index, 2, 4, 4)
        };
        let wave = get_instrument_bits(song, index, 3, 0, 8);
        let play_mode = WavePlayMode::from_bytes(song, index)?;

        let length = if format_version >= 7 {
            0xF - get_instrument_bits(song, index, 10, 0, 4)
        } else if format_version == 6 {
            get_instrument_bits(song, index, 10, 0, 4)
        } else {
            get_instrument_bits(song, index, 14, 4, 4)
        };

        let loop_pos = {
            let byte = get_instrument_bits(song, index, 2, 0, 4);

            if format_version >= 9 {
                byte & 0xF
            } else {
                (byte & 0xF) ^ 0x0F
            }
        };

        let repeat = {
            let byte = get_instrument_bits(song, index, 2, 0, 4);

            if format_version >= 9 {
                (byte & 0xF) ^ 0xF
            } else {
                byte & 0xF
            }
        };

        let speed = {
            let raw_speed = if format_version >= 7 {
                get_instrument_bits(song, index, 11, 0, 8).wrapping_add(3)
            } else if format_version == 6 {
                get_instrument_bits(song, index, 11, 0, 8)
            } else {
                get_instrument_bits(song, index, 14, 0, 4)
            };

            // Speed is stored as starting at 0, but displayed as starting at 1, so add 1
            raw_speed + 1
        };

        Ok(Self {
            volume,
            synth,
            wave,
            play_mode,
            length,
            loop_pos,
            repeat,
            speed,
        })
    }
}

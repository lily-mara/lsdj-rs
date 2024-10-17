//! The constants in this module store the offsets into a SongMemory where
//! various fields are stored. This module is not intended to be publicly used.
//!
//! Constant values are taken from song_offsets.h in liblsdj
//! https://github.com/stijnfrishert/libLSDJ/blob/6023c4e48ad8280abacfddba60f2689e2442d79c/liblsdj/src/song_offsets.h#L39

pub(crate) const FORMAT_VERSION_OFFSET: usize = 0x7FFF;

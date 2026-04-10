//! Dataset loading for C-MAPSS and N-CMAPSS.
//!
//! This module is gated behind `feature = "std"` because it requires
//! file I/O, heap allocation, and string processing.

pub mod cmapss;
pub mod ncmapss;

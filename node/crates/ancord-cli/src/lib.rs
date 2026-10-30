//! Knihovna CLI klienta: síťová a online logika je v `ancord-core` (sdílí ji grafický klient), tady zbývá
//! souborové úložiště pro příkazovou řádku.

pub use ancord_core::net::*;
pub use ancord_core::online;

mod files;
pub use files::*;

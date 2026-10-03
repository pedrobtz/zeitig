//! Rust side of zudate. Every `#[savvy]` function is stateless: it receives
//! whole R vectors, builds `jiff` values element by element and returns whole
//! R vectors (see design.md, "Architecture").

mod tz;

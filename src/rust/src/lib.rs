//! Rust side of zietig. Every `#[savvy]` function is stateless: it receives
//! whole R vectors, builds `jiff` values element by element and returns whole
//! R vectors (see design.md, "Architecture").

mod arith;
mod civil;
mod cols;
mod duration;
mod format;
mod opts;
mod tz;
mod zoned;

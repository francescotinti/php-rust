//! Library surface of the CLI crate: exposes the oracle-pinned cli-server
//! SAPI (`server`, WP-4/5/6) and its mime table so sibling binaries
//! (`php-server`) reuse the exact `phpr -S` implementation instead of
//! carrying their own. The `phpr` binary keeps its private module copies in
//! `main.rs`; both targets compile the same source files.

pub mod mime;
pub mod server;

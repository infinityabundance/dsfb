//! Must fail to compile: `prepare` would let a caller hold a raw
//! `Statement` that escapes the allow-list.
use dsfb_database::live::ReadOnlyPgConn;

fn _must_not_compile(c: &ReadOnlyPgConn) {
    let _ = c.prepare("SELECT 1");
}

fn main() {}

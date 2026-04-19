//! Must fail to compile: `transaction` would allow arbitrary SQL to
//! run inside a transaction block.
use dsfb_database::live::ReadOnlyPgConn;

fn _must_not_compile(c: &mut ReadOnlyPgConn) {
    let _ = c.transaction();
}

fn main() {}

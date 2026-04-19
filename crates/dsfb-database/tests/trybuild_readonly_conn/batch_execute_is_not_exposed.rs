//! Must fail to compile: `batch_execute` allows multi-statement SQL.
use dsfb_database::live::ReadOnlyPgConn;

fn _must_not_compile(c: &ReadOnlyPgConn) {
    let _ = c.batch_execute("SELECT 1; DROP TABLE users");
}

fn main() {}

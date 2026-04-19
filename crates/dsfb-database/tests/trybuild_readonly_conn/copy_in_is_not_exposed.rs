//! Must fail to compile: `copy_in` could bulk-load arbitrary rows.
use dsfb_database::live::ReadOnlyPgConn;

fn _must_not_compile(c: &ReadOnlyPgConn) {
    let _ = c.copy_in::<_, Vec<u8>>("COPY users FROM STDIN");
}

fn main() {}

//! Must fail to compile: `execute` is a `tokio_postgres::Client` method
//! that must NOT be reachable through `ReadOnlyPgConn`.
use dsfb_database::live::ReadOnlyPgConn;

fn _must_not_compile(c: &ReadOnlyPgConn) {
    // No public `execute` on the wrapper. If a future edit accidentally
    // adds one (or a `Deref<Target = Client>`), this line will start to
    // compile and the trybuild test will fire.
    let _ = c.execute("DROP TABLE users", &[]);
}

fn main() {}

//! Type-level data-diode proof.
//!
//! This test is the compile-time half of the three-layer read-only
//! contract described in [`dsfb_database::live`] module docs: it
//! asserts that `ReadOnlyPgConn` does not re-export any of
//! `tokio_postgres::Client`'s mutating methods. A code audit of the
//! live module confirms the private field is the only handle on a
//! `Client`, and this test confirms that the public surface exposes
//! no path to that handle.
//!
//! The `trybuild` crate compiles each `.rs` file in `trybuild_readonly_conn/`
//! and asserts it fails to compile. If a future edit to
//! [`dsfb_database::live::readonly_conn`] accidentally adds a
//! `Deref<Target = Client>`, an `as_client(&self)` accessor, or a
//! re-exported `execute`/`prepare`/`transaction` method, these
//! fixtures will start to compile and the test will fire.

#![cfg(feature = "live-postgres")]

#[test]
fn readonly_conn_surface_rejects_mutating_calls() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild_readonly_conn/*.rs");
}

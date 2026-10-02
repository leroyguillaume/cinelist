fn main() {
    // Rebuild whenever an embedded file changes: `sqlx::migrate!` and
    // `include_str!` expand at compile time.
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=migrations");
    println!("cargo::rerun-if-changed=queries");
    println!("cargo::rerun-if-changed=static");
}

#[test]
fn rust_package_uses_edition_2024() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    assert!(manifest.lines().any(|line| line == r#"edition = "2024""#));
}

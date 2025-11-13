use assert_cmd::{Command, cargo};
use predicates::prelude::*;

#[test]
fn test_read_file() -> Result<(), Box<dyn std::error::Error>> {
    let expected = r#"Do you have what it takes to be an engineer at TheStartup™?
Are you willing to work 80 hours a week in hopes that your 0.001% equity is worth something?
Can you say "synergy" and "democratize" with a straight face?
Are you prepared to eat top ramen at your desk 3 meals a day?
end"#;
    let mut cmd = Command::new(cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(expected));
    Ok(())
}
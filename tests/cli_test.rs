use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn octav() -> Command {
    Command::cargo_bin("octav").unwrap()
}

#[test]
fn test_help() {
    octav()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "CLI for the Octav crypto portfolio API",
        ));
}

#[test]
fn test_portfolio_help() {
    octav()
        .args(["portfolio", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio endpoints"));
}

#[test]
fn test_no_api_key_error() {
    // Ensure OCTAV_API_KEY is not set for this test
    octav()
        .env_remove("OCTAV_API_KEY")
        .arg("credits")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"type\": \"config\""));
}

#[test]
fn test_validation_error_invalid_address() {
    octav()
        .env_remove("OCTAV_API_KEY")
        .args([
            "--api-key",
            "test",
            "portfolio",
            "get",
            "--addresses",
            "invalid",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"type\": \"validation\""))
        .stdout(predicate::str::contains("Invalid address format"));
}

#[test]
fn test_validation_error_too_many_addresses() {
    let addrs = ["0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68"; 11].join(",");
    octav()
        .env_remove("OCTAV_API_KEY")
        .args([
            "--api-key",
            "test",
            "portfolio",
            "get",
            "--addresses",
            &addrs,
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Maximum 10 addresses"));
}

#[test]
fn test_validation_error_bad_date() {
    octav()
        .env_remove("OCTAV_API_KEY")
        .args([
            "--api-key",
            "test",
            "portfolio",
            "token-overview",
            "--addresses",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68",
            "--date",
            "not-a-date",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Invalid date format"));
}

#[test]
fn test_auth_set_key_and_show() {
    // Use a temp HOME to avoid polluting real config
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    octav()
        .env("HOME", home.to_str().unwrap())
        .env_remove("OCTAV_API_KEY")
        .args(["auth", "set-key", "oct_test123456789"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API key saved"));

    octav()
        .env("HOME", home.to_str().unwrap())
        .env_remove("OCTAV_API_KEY")
        .args(["auth", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config_file"))
        .stdout(predicate::str::contains("oct_***...789"));
}

#[test]
fn test_transactions_help() {
    octav()
        .args(["transactions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Transaction endpoints"));
}

#[test]
fn test_historical_help() {
    octav()
        .args(["historical", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Historical data endpoints"));
}

#[test]
fn test_agent_help() {
    octav()
        .args(["agent", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent endpoints"));
}

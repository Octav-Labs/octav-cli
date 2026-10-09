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
fn test_version() {
    octav()
        .arg("--version")
        .assert()
        .success()
        .stdout(format!("octav {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_update_help() {
    octav()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Update octav to the latest release",
        ))
        .stdout(predicate::str::contains("--check"));
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
    // Ensure no API key is available: no env var, and a temp HOME so a real
    // ~/.octav/config.json isn't picked up
    let tmp = tempfile::tempdir().unwrap();
    octav()
        .env("HOME", tmp.path().to_str().unwrap())
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
        .stdout(predicate::str::contains("Agent endpoints"))
        .stdout(predicate::str::contains("nav"))
        .stdout(predicate::str::contains("chains"));
}

#[test]
fn test_new_command_help() {
    for (args, expected) in [
        (vec!["chains", "--help"], "Supported chains and protocols"),
        (
            vec!["addressbook", "--help"],
            "Manage your saved address book",
        ),
        (vec!["bundles", "--help"], "Manage bundles"),
        (vec!["virtual-users", "--help"], "Virtual user endpoints"),
        (vec!["approvals", "--help"], "Get token approvals"),
        (
            vec!["contract-protocol", "--help"],
            "Resolve a contract address",
        ),
        (
            vec!["portfolio", "at-block", "--help"],
            "valued at a specific block",
        ),
    ] {
        octav()
            .args(&args)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}

/// Run with a dummy key; these all fail validation before any request is sent
fn octav_offline(args: &[&str]) -> assert_cmd::assert::Assert {
    octav()
        .env_remove("OCTAV_API_KEY")
        .args(["--api-key", "test"])
        .args(args)
        .assert()
}

#[test]
fn test_transactions_type_flag_and_alias() {
    // `--type` used to be unreachable (clap generated `--tx-type`)
    for flag in ["--type", "--tx-type"] {
        octav_offline(&[
            "transactions",
            "get",
            "--addresses",
            "invalid",
            flag,
            "SWAP",
        ])
        .failure()
        .stdout(predicate::str::contains("\"type\": \"validation\""));
    }
}

#[test]
fn test_transactions_invalid_interacting_address() {
    octav_offline(&[
        "transactions",
        "get",
        "--addresses",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68",
        "--interacting-address",
        "nope",
    ])
    .failure()
    .stdout(predicate::str::contains("Invalid address format: 'nope'"));
}

#[test]
fn test_at_block_requires_evm_address() {
    octav_offline(&[
        "portfolio",
        "at-block",
        "--address",
        "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
        "--chain",
        "ethereum",
        "--block",
        "1",
    ])
    .failure()
    .stdout(predicate::str::contains("Must be an EVM"));
}

#[test]
fn test_addressbook_label_requires_single_address() {
    octav_offline(&[
        "addressbook",
        "add",
        "--addresses",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68,0x000000000000000000000000000000000000dEaD",
        "--label",
        "Main",
    ])
    .failure()
    .stdout(predicate::str::contains("single address"));
}

#[test]
fn test_destructive_commands_require_yes() {
    let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68";
    for args in [
        vec!["addressbook", "remove", "--address", addr],
        vec!["addressbook", "rename", "--address", addr, "--label", "x"],
        vec!["bundles", "delete", "--id", "abc123"],
        vec!["bundles", "rename", "--id", "abc123", "--name", "x"],
        vec![
            "bundles",
            "remove-address",
            "--id",
            "abc123",
            "--address",
            addr,
        ],
    ] {
        // Fails before any request is sent, so the dummy key is never used
        octav_offline(&args)
            .failure()
            .stdout(predicate::str::contains(
                "\"type\": \"confirmation_required\"",
            ))
            .stdout(predicate::str::contains("Re-run with --yes"));
    }
}

#[test]
fn test_write_commands_marked_in_help() {
    for group in ["addressbook", "bundles"] {
        octav()
            .args([group, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[destructive; requires --yes]"))
            .stdout(predicate::str::contains("[overwrites; requires --yes]"))
            .stdout(predicate::str::contains("[writes to your account]"));
    }
    octav()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "some subcommands modify your account",
        ));
}

#[test]
fn test_bundle_id_cannot_escape_path() {
    octav_offline(&["bundles", "get", "--id", "../credits"])
        .failure()
        .stdout(predicate::str::contains("Invalid bundle ID"));
}

#[test]
fn test_virtual_users_requires_virtual_addresses() {
    octav_offline(&[
        "virtual-users",
        "portfolio",
        "--addresses",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68",
    ])
    .failure()
    .stdout(predicate::str::contains("Must be virtual:<id>"));
}

#[test]
fn test_approvals_limit_range() {
    octav_offline(&[
        "approvals",
        "--address",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68",
        "--chain",
        "ethereum",
        "--limit",
        "101",
    ])
    .failure()
    .stderr(predicate::str::contains("101 is not in 1..=100"));
}

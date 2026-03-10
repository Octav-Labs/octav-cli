mod cli;
mod client;
mod commands;
mod config;
mod error;
mod tui;
mod types;
mod validation;

use std::process;

use clap::Parser;
use serde_json::Value;

use cli::{
    AgentCommand, AuthCommand, Cli, Command, HistoricalCommand, PortfolioCommand,
    TransactionsCommand,
};
use client::OctavClient;
use error::OctavError;

fn resolve_api_key(cli_key: Option<&str>) -> Result<String, OctavError> {
    if let Some(key) = cli_key {
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }
    let cfg = config::load_config()?;
    Ok(cfg.api_key)
}

fn output(value: &Value, raw: bool) {
    let json = if raw {
        serde_json::to_string(value).unwrap()
    } else {
        serde_json::to_string_pretty(value).unwrap()
    };
    println!("{}", json);
}

fn run(cli: Cli) -> Result<Value, OctavError> {
    let raw = cli.raw;

    match cli.command {
        Command::Dashboard { addresses } => {
            let api_key = resolve_api_key(cli.api_key.as_deref())?;
            commands::dashboard::run(&api_key, &addresses)?;
            return Ok(Value::Null);
        }

        Command::Auth { command } => match command {
            AuthCommand::SetKey { key } => commands::auth::set_key(&key),
            AuthCommand::Show => {
                let env_key = std::env::var("OCTAV_API_KEY").ok();
                let flag_key =
                    if cli.api_key.is_some() && env_key.as_deref() != cli.api_key.as_deref() {
                        cli.api_key.as_deref()
                    } else {
                        None
                    };
                commands::auth::show(flag_key, env_key.as_deref())
            }
        },

        command => {
            let api_key = resolve_api_key(cli.api_key.as_deref())?;
            let client = OctavClient::new(api_key);

            match command {
                Command::Portfolio { command } => match command {
                    PortfolioCommand::Get { addresses } => {
                        commands::portfolio::get(&client, &addresses, raw)
                    }
                    PortfolioCommand::Wallet { addresses } => {
                        commands::portfolio::wallet(&client, &addresses, raw)
                    }
                    PortfolioCommand::Nav {
                        addresses,
                        currency,
                    } => commands::portfolio::nav(&client, &addresses, &currency.to_string()),
                    PortfolioCommand::TokenOverview { addresses, date } => {
                        commands::portfolio::token_overview(&client, &addresses, &date)
                    }
                },

                Command::Transactions { command } => match command {
                    TransactionsCommand::Get {
                        addresses,
                        chain,
                        tx_type,
                        start_date,
                        end_date,
                        offset,
                        limit,
                    } => commands::transactions::get(
                        &client,
                        &addresses,
                        chain.as_deref(),
                        tx_type.as_deref(),
                        start_date.as_deref(),
                        end_date.as_deref(),
                        offset,
                        limit,
                    ),
                    TransactionsCommand::Sync { addresses } => {
                        commands::transactions::sync(&client, &addresses)
                    }
                },

                Command::Historical { command } => match command {
                    HistoricalCommand::Get { addresses, date } => {
                        commands::historical::get(&client, &addresses, &date)
                    }
                    HistoricalCommand::SubscribeSnapshot {
                        addresses,
                        description,
                    } => commands::historical::subscribe_snapshot(
                        &client,
                        &addresses,
                        description.as_deref(),
                    ),
                },

                Command::Status { addresses } => commands::metadata::status(&client, &addresses),
                Command::Credits => commands::metadata::credits(&client),

                Command::Airdrop { address } => commands::specialized::airdrop(&client, &address),
                Command::Polymarket { address } => {
                    commands::specialized::polymarket(&client, &address)
                }

                Command::Agent { command } => match command {
                    AgentCommand::Wallet { addresses } => {
                        commands::specialized::agent_wallet(&client, &addresses, raw)
                    }
                    AgentCommand::Portfolio { addresses } => {
                        commands::specialized::agent_portfolio(&client, &addresses, raw)
                    }
                },

                Command::Auth { .. } | Command::Dashboard { .. } => unreachable!(),
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let raw = cli.raw;

    match run(cli) {
        Ok(Value::Null) => {}
        Ok(value) => output(&value, raw),
        Err(e) => {
            let json = e.to_json();
            let out = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", out);
            process::exit(1);
        }
    }
}

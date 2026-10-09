mod cli;
mod client;
mod commands;
mod config;
mod error;
mod tui;
mod types;
mod update;
mod validation;

use std::process;

use clap::Parser;
use serde_json::Value;

use cli::{
    AddressbookCommand, AgentCommand, AuthCommand, BundlesCommand, ChainsCommand, Cli, Command,
    HistoricalCommand, PortfolioCommand, TransactionsCommand, VirtualUsersCommand,
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

        Command::Update { check } => {
            if check {
                update::check()
            } else {
                update::update()
            }
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
                    PortfolioCommand::Get { addresses, options } => {
                        commands::portfolio::get(&client, &addresses, &options, raw)
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
                    PortfolioCommand::AtBlock {
                        address,
                        chain,
                        block,
                    } => commands::portfolio::at_block(&client, &address, &chain, block, raw),
                },

                Command::Transactions { command } => match command {
                    TransactionsCommand::Get {
                        addresses,
                        filters,
                        offset,
                        limit,
                    } => commands::transactions::get(&client, &addresses, &filters, offset, limit),
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
                Command::Approvals {
                    address,
                    chain,
                    limit,
                    cursor,
                } => commands::specialized::approvals(
                    &client,
                    &address,
                    &chain,
                    limit,
                    cursor.as_deref(),
                ),

                Command::Chains { command } => match command {
                    ChainsCommand::List => commands::metadata::chains(&client),
                    ChainsCommand::Protocols { chain, page, limit } => {
                        commands::metadata::chain_protocols(&client, &chain, page, limit)
                    }
                },
                Command::ContractProtocol { contract, chain } => {
                    commands::metadata::contract_protocol(&client, &contract, chain.as_deref())
                }

                Command::Addressbook { command } => match command {
                    AddressbookCommand::List => commands::addressbook::list(&client),
                    AddressbookCommand::Add { addresses, label } => {
                        commands::addressbook::add(&client, &addresses, label.as_deref())
                    }
                    AddressbookCommand::Rename {
                        address,
                        label,
                        yes,
                    } => commands::addressbook::rename(&client, &address, &label, yes),
                    AddressbookCommand::Remove { address, yes } => {
                        commands::addressbook::remove(&client, &address, yes)
                    }
                },

                Command::Bundles { command } => match command {
                    BundlesCommand::List => commands::bundles::list(&client),
                    BundlesCommand::Get { id } => commands::bundles::get(&client, &id),
                    BundlesCommand::Create { name, addresses } => {
                        commands::bundles::create(&client, &name, &addresses)
                    }
                    BundlesCommand::Rename { id, name, yes } => {
                        commands::bundles::rename(&client, &id, &name, yes)
                    }
                    BundlesCommand::Delete { id, yes } => {
                        commands::bundles::delete(&client, &id, yes)
                    }
                    BundlesCommand::AddAddress { id, address } => {
                        commands::bundles::add_address(&client, &id, &address)
                    }
                    BundlesCommand::RemoveAddress { id, address, yes } => {
                        commands::bundles::remove_address(&client, &id, &address, yes)
                    }
                },

                Command::VirtualUsers { command } => match command {
                    VirtualUsersCommand::List => commands::virtual_users::list(&client),
                    VirtualUsersCommand::Portfolio {
                        addresses,
                        aggregated,
                        options,
                    } => commands::virtual_users::portfolio(
                        &client, &addresses, aggregated, &options, raw,
                    ),
                },

                Command::Agent { command } => match command {
                    AgentCommand::Wallet { addresses } => {
                        commands::specialized::agent_wallet(&client, &addresses, raw)
                    }
                    AgentCommand::Portfolio { addresses } => {
                        commands::specialized::agent_portfolio(&client, &addresses, raw)
                    }
                    AgentCommand::Nav {
                        addresses,
                        currency,
                    } => {
                        commands::specialized::agent_nav(&client, &addresses, &currency.to_string())
                    }
                    AgentCommand::Status { addresses } => {
                        commands::specialized::agent_status(&client, &addresses)
                    }
                    AgentCommand::Chains => commands::specialized::agent_chains(&client),
                },

                Command::Auth { .. } | Command::Dashboard { .. } | Command::Update { .. } => {
                    unreachable!()
                }
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let raw = cli.raw;

    // The dashboard owns the terminal, and `update` checks for itself
    let update_check = match cli.command {
        Command::Dashboard { .. } | Command::Update { .. } => None,
        _ => update::start_background_check(),
    };

    let code = match run(cli) {
        Ok(Value::Null) => 0,
        Ok(value) => {
            output(&value, raw);
            0
        }
        Err(e) => {
            let json = e.to_json();
            let out = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", out);
            1
        }
    };

    if let Some(check) = update_check {
        check.finish();
    }
    process::exit(code);
}

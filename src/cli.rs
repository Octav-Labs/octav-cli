use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "octav", about = "CLI for the Octav crypto portfolio API")]
pub struct Cli {
    /// API key (overrides OCTAV_API_KEY env and config file)
    #[arg(long, global = true, env = "OCTAV_API_KEY")]
    pub api_key: Option<String>,

    /// Output compact JSON without portfolio field stripping
    #[arg(long, global = true)]
    pub raw: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage API key authentication
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },

    /// Portfolio endpoints
    Portfolio {
        #[command(subcommand)]
        command: PortfolioCommand,
    },

    /// Transaction endpoints
    Transactions {
        #[command(subcommand)]
        command: TransactionsCommand,
    },

    /// Historical data endpoints
    Historical {
        #[command(subcommand)]
        command: HistoricalCommand,
    },

    /// Check sync status for addresses (FREE)
    Status {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },

    /// Check credit balance (FREE)
    Credits,

    /// Check Solana airdrop eligibility
    Airdrop {
        /// Wallet address
        #[arg(long, required = true)]
        address: String,
    },

    /// Get Polymarket positions
    Polymarket {
        /// Wallet address
        #[arg(long, required = true)]
        address: String,
    },

    /// Agent endpoints (x402 payment)
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
}

#[derive(Subcommand)]
pub enum AuthCommand {
    /// Store API key in ~/.octav/config.json
    SetKey {
        /// The API key to store
        key: String,
    },
    /// Show current API key source and masked value
    Show,
}

#[derive(Subcommand)]
pub enum PortfolioCommand {
    /// Get full portfolio including DeFi positions
    Get {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
    /// Get wallet holdings only
    Wallet {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
    /// Get net asset value
    Nav {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Currency for NAV calculation
        #[arg(long, default_value = "USD")]
        currency: Currency,
    },
    /// Get token overview for a specific date
    TokenOverview {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Date in YYYY-MM-DD format
        #[arg(long, required = true)]
        date: String,
    },
}

#[derive(Subcommand)]
pub enum TransactionsCommand {
    /// Get transactions
    Get {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Filter by chain
        #[arg(long)]
        chain: Option<String>,
        /// Filter by transaction type
        #[arg(long, name = "type")]
        tx_type: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end_date: Option<String>,
        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: u32,
        /// Results per page (max 250)
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Trigger transaction sync
    Sync {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum HistoricalCommand {
    /// Get historical portfolio data for a date
    Get {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Date in YYYY-MM-DD format
        #[arg(long, required = true)]
        date: String,
    },
    /// Subscribe to daily portfolio snapshots
    SubscribeSnapshot {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Optional description for the subscription
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Get wallet data via x402 payment
    Wallet {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
    /// Get portfolio data via x402 payment
    Portfolio {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
#[allow(clippy::upper_case_acronyms)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CNY,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CNY => write!(f, "CNY"),
        }
    }
}

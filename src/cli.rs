use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "octav",
    version,
    about = "CLI for the Octav crypto portfolio API"
)]
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

    /// Get token approvals for a wallet on one chain
    Approvals {
        /// EVM wallet address
        #[arg(long, required = true)]
        address: String,
        /// Chain key (e.g. ethereum, arbitrum, base)
        #[arg(long, required = true)]
        chain: String,
        /// Results per page (1-100)
        #[arg(long, default_value = "25", value_parser = clap::value_parser!(u32).range(1..=100))]
        limit: u32,
        /// Cursor from a previous response, to fetch the next page
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Supported chains and protocols (FREE)
    Chains {
        #[command(subcommand)]
        command: ChainsCommand,
    },

    /// Resolve a contract address to its DeFi protocol (5 credits, refunded if not found)
    ContractProtocol {
        /// Contract address (EVM or Solana)
        #[arg(long, required = true)]
        contract: String,
        /// Chain key; omit to search all chains
        #[arg(long)]
        chain: Option<String>,
    },

    /// Manage your saved address book (some subcommands modify your account)
    ///
    /// `add`, `rename` and `remove` change data on your Octav account. `rename` and
    /// `remove` overwrite or delete data and require --yes.
    Addressbook {
        #[command(subcommand)]
        command: AddressbookCommand,
    },

    /// Manage bundles, named groups of address book entries (some subcommands modify your account)
    ///
    /// `create`, `rename`, `delete`, `add-address` and `remove-address` change data on your
    /// Octav account. `rename`, `delete` and `remove-address` overwrite or delete data and
    /// require --yes.
    Bundles {
        #[command(subcommand)]
        command: BundlesCommand,
    },

    /// Virtual user endpoints (requires Pro)
    VirtualUsers {
        #[command(subcommand)]
        command: VirtualUsersCommand,
    },

    /// Interactive portfolio dashboard
    Dashboard {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },

    /// Agent endpoints (x402 payment)
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },

    /// Update octav to the latest release
    ///
    /// Replaces the running binary with the latest GitHub release. Installs made with
    /// cargo or built from source are not replaced; the command prints what to run instead.
    Update {
        /// Only check whether a newer version is available
        #[arg(long)]
        check: bool,
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
        #[command(flatten)]
        options: PortfolioOptions,
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
    /// Get portfolio valued at a specific block (requires the Portfolio at Block add-on)
    AtBlock {
        /// EVM wallet address
        #[arg(long, required = true)]
        address: String,
        /// Chain the block belongs to (ethereum, linea or monad)
        #[arg(long, required = true)]
        chain: String,
        /// Block number
        #[arg(long, required = true, value_parser = clap::value_parser!(u64).range(1..))]
        block: u64,
    },
}

/// Options shared by endpoints that return a full portfolio
#[derive(Args, Default)]
pub struct PortfolioOptions {
    /// Return cached data immediately instead of waiting for a fresh sync
    #[arg(long)]
    pub no_wait_for_sync: bool,
    /// Include blockchain explorer URLs for assets and transactions
    #[arg(long)]
    pub explorer_urls: bool,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // parsed once per run; size doesn't matter
pub enum TransactionsCommand {
    /// Get transactions
    Get {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        #[command(flatten)]
        filters: TransactionFilters,
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

#[derive(Args, Default)]
pub struct TransactionFilters {
    /// Filter by chain keys, comma-separated (see `octav chains list`)
    #[arg(long, value_delimiter = ',')]
    pub chain: Vec<String>,
    /// Filter by transaction types, comma-separated (e.g. SWAP,DEPOSIT)
    #[arg(long = "type", alias = "tx-type", value_delimiter = ',')]
    pub tx_type: Vec<String>,
    /// Filter by protocol keys, comma-separated
    #[arg(long, value_delimiter = ',')]
    pub protocol: Vec<String>,
    /// Filter by counterparty addresses, comma-separated
    #[arg(long, value_delimiter = ',')]
    pub interacting_address: Vec<String>,
    /// Full-text search over token symbols, names and addresses
    #[arg(long)]
    pub search: Option<String>,
    /// Filter by NFT token ID
    #[arg(long)]
    pub token_id: Option<String>,
    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    pub start_date: Option<String>,
    /// End date (YYYY-MM-DD)
    #[arg(long)]
    pub end_date: Option<String>,
    /// Sort order by timestamp [default: desc]
    #[arg(long, ignore_case = true)]
    pub sort: Option<SortOrder>,
    /// Exclude spam transactions
    #[arg(long)]
    pub hide_spam: bool,
    /// Exclude dust transactions
    #[arg(long)]
    pub hide_dust: bool,
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
pub enum ChainsCommand {
    /// List supported chains (FREE)
    List,
    /// List protocols on a chain (FREE)
    Protocols {
        /// Chain key (e.g. ethereum, solana, arbitrum)
        #[arg(long, required = true)]
        chain: String,
        /// Page number
        #[arg(long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..))]
        page: u32,
        /// Protocols per page (1-100)
        #[arg(long, default_value = "20", value_parser = clap::value_parser!(u32).range(1..=100))]
        limit: u32,
    },
}

#[derive(Subcommand)]
pub enum AddressbookCommand {
    /// List saved addresses
    List,
    /// Add addresses, up to 100 [writes to your account]
    ///
    /// Addresses already saved keep their existing label.
    Add {
        /// Comma-separated addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Label for the address (only when adding a single address)
        #[arg(long)]
        label: Option<String>,
    },
    /// Replace the label of a saved address [overwrites; requires --yes]
    Rename {
        /// Saved address
        #[arg(long, required = true)]
        address: String,
        /// New label (pass "" to clear it)
        #[arg(long, required = true)]
        label: String,
        /// Confirm overwriting the current label
        #[arg(long)]
        yes: bool,
    },
    /// Remove a saved address [destructive; requires --yes]
    Remove {
        /// Saved address
        #[arg(long, required = true)]
        address: String,
        /// Confirm removing the address from your account
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum BundlesCommand {
    /// List bundles
    List,
    /// Get one bundle
    Get {
        /// Bundle ID
        #[arg(long, required = true)]
        id: String,
    },
    /// Create a bundle from addresses already in the address book [writes to your account]
    Create {
        /// Bundle name (unique per account)
        #[arg(long, required = true)]
        name: String,
        /// Comma-separated addresses (1-100)
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
    /// Rename a bundle [overwrites; requires --yes]
    Rename {
        /// Bundle ID
        #[arg(long, required = true)]
        id: String,
        /// New name
        #[arg(long, required = true)]
        name: String,
        /// Confirm overwriting the current name
        #[arg(long)]
        yes: bool,
    },
    /// Delete a bundle [destructive; requires --yes]
    ///
    /// The bundle's addresses stay in the address book.
    Delete {
        /// Bundle ID
        #[arg(long, required = true)]
        id: String,
        /// Confirm deleting the bundle from your account
        #[arg(long)]
        yes: bool,
    },
    /// Add an address book entry to a bundle [writes to your account]
    AddAddress {
        /// Bundle ID
        #[arg(long, required = true)]
        id: String,
        /// Address (must already be in the address book)
        #[arg(long, required = true)]
        address: String,
    },
    /// Remove an address from a bundle [destructive; requires --yes]
    ///
    /// The address stays in the address book.
    RemoveAddress {
        /// Bundle ID
        #[arg(long, required = true)]
        id: String,
        /// Address to remove
        #[arg(long, required = true)]
        address: String,
        /// Confirm removing the address from the bundle
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum VirtualUsersCommand {
    /// List virtual users on your account
    List,
    /// Get portfolio for virtual users
    Portfolio {
        /// Comma-separated virtual user addresses (virtual:<id>)
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Return a single portfolio aggregated across all virtual users
        #[arg(long)]
        aggregated: bool,
        #[command(flatten)]
        options: PortfolioOptions,
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
    /// Get net asset value via x402 payment
    Nav {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
        /// Currency for NAV calculation
        #[arg(long, default_value = "USD")]
        currency: Currency,
    },
    /// Get sync status via x402 payment
    Status {
        /// Comma-separated wallet addresses
        #[arg(long, required = true, value_delimiter = ',')]
        addresses: Vec<String>,
    },
    /// List supported chains via x402 payment
    Chains,
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

#[derive(Debug, Clone, ValueEnum)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

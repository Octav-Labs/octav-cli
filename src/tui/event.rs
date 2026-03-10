use crate::tui::data::*;

pub enum DataEvent {
    CreditsLoaded(f64),
    NavLoaded(NavData),
    PortfolioLoaded {
        protocols: Vec<ProtocolData>,
        holdings: Vec<HoldingRow>,
        chains: Vec<ChainAllocation>,
    },
    TransactionsLoaded {
        items: Vec<TransactionItem>,
        total: u64,
    },
    ImageLoaded {
        url: String,
        image_data: image::DynamicImage,
    },
    FetchError(String),
    LoadingStarted,
}

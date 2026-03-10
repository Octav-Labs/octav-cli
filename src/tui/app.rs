use std::collections::HashMap;
use std::sync::mpsc;

use ratatui::widgets::TableState;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

use crate::tui::data::*;
use crate::tui::event::DataEvent;
use crate::tui::images::ImageCache;

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Overview,
    Holdings,
    Protocols,
    Transactions,
}

impl Screen {
    pub fn label(&self) -> &'static str {
        match self {
            Screen::Overview => "Overview",
            Screen::Holdings => "Holdings",
            Screen::Protocols => "Protocols",
            Screen::Transactions => "Transactions",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Screen::Overview => 0,
            Screen::Holdings => 1,
            Screen::Protocols => 2,
            Screen::Transactions => 3,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Screen::Overview,
            1 => Screen::Holdings,
            2 => Screen::Protocols,
            3 => Screen::Transactions,
            _ => Screen::Overview,
        }
    }

    pub fn all() -> &'static [Screen] {
        &[
            Screen::Overview,
            Screen::Holdings,
            Screen::Protocols,
            Screen::Transactions,
        ]
    }
}

#[derive(Clone)]
pub enum ProtocolLevel {
    List,
    Detail {
        protocol_index: usize,
    },
    Assets {
        protocol_index: usize,
        chain_index: usize,
        position_index: usize,
    },
}

pub enum LoadState {
    Idle,
    Loading,
    Ready,
    Error(#[allow(dead_code)] String),
}

pub struct App {
    pub active_screen: Screen,
    pub should_quit: bool,

    pub addresses: Vec<String>,
    pub load_state: LoadState,

    // Data
    pub nav: Option<NavData>,
    pub protocols: Vec<ProtocolData>,
    pub holdings: Vec<HoldingRow>,
    pub chain_allocations: Vec<ChainAllocation>,
    pub transactions: Vec<TransactionItem>,
    pub transactions_total: u64,
    pub credits: Option<f64>,

    // Table states
    pub holdings_table: TableState,
    pub transactions_table: TableState,

    // Protocol drill-down
    pub protocol_level: ProtocolLevel,
    pub protocol_list_state: TableState,
    pub protocol_detail_state: TableState,
    pub protocol_assets_state: TableState,

    // Images
    pub picker: Picker,
    pub token_images: HashMap<String, StatefulProtocol>,
    pub image_cache: ImageCache,
    pub event_tx: mpsc::Sender<DataEvent>,

    // Status
    pub last_error: Option<String>,
    pub tick_count: u64,
}

impl App {
    pub fn new(addresses: Vec<String>, picker: Picker, event_tx: mpsc::Sender<DataEvent>) -> Self {
        Self {
            active_screen: Screen::Overview,
            should_quit: false,
            addresses,
            load_state: LoadState::Idle,
            nav: None,
            protocols: Vec::new(),
            holdings: Vec::new(),
            chain_allocations: Vec::new(),
            transactions: Vec::new(),
            transactions_total: 0,
            credits: None,
            holdings_table: TableState::default(),
            transactions_table: TableState::default(),
            protocol_level: ProtocolLevel::List,
            protocol_list_state: TableState::default(),
            protocol_detail_state: TableState::default(),
            protocol_assets_state: TableState::default(),
            picker,
            token_images: HashMap::new(),
            image_cache: ImageCache::new(),
            event_tx,
            last_error: None,
            tick_count: 0,
        }
    }

    pub fn next_screen(&mut self) {
        let idx = (self.active_screen.index() + 1) % 4;
        self.active_screen = Screen::from_index(idx);
    }

    pub fn prev_screen(&mut self) {
        let idx = if self.active_screen.index() == 0 {
            3
        } else {
            self.active_screen.index() - 1
        };
        self.active_screen = Screen::from_index(idx);
    }

    pub fn handle_data_event(&mut self, event: DataEvent) {
        match event {
            DataEvent::LoadingStarted => {
                self.load_state = LoadState::Loading;
                self.last_error = None;
            }
            DataEvent::CreditsLoaded(credits) => {
                self.credits = Some(credits);
            }
            DataEvent::NavLoaded(nav) => {
                self.nav = Some(nav);
            }
            DataEvent::PortfolioLoaded {
                protocols,
                holdings,
                chains,
            } => {
                self.protocols = protocols;
                self.holdings = holdings;
                self.chain_allocations = chains;
                self.load_state = LoadState::Ready;
                if !self.holdings.is_empty() && self.holdings_table.selected().is_none() {
                    self.holdings_table.select(Some(0));
                }
                if !self.protocols.is_empty() && self.protocol_list_state.selected().is_none() {
                    self.protocol_list_state.select(Some(0));
                }
                // Kick off image downloads for holdings
                let urls: Vec<String> = self
                    .holdings
                    .iter()
                    .map(|h| h.image_url.clone())
                    .filter(|u| !u.is_empty())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                self.image_cache.fetch_images(urls, self.event_tx.clone());
            }
            DataEvent::TransactionsLoaded { items, total } => {
                self.transactions = items;
                self.transactions_total = total;
                if !self.transactions.is_empty() && self.transactions_table.selected().is_none() {
                    self.transactions_table.select(Some(0));
                }
            }
            DataEvent::ImageLoaded { url, image_data } => {
                let protocol = self.picker.new_resize_protocol(image_data);
                self.token_images.insert(url, protocol);
            }
            DataEvent::FetchError(msg) => {
                self.last_error = Some(msg.clone());
                self.load_state = LoadState::Error(msg);
            }
        }
    }

    pub fn scroll_down(&mut self) {
        match self.active_screen {
            Screen::Holdings => {
                let len = self.holdings.len();
                scroll_table_down(&mut self.holdings_table, len);
            }
            Screen::Transactions => {
                let len = self.transactions.len();
                scroll_table_down(&mut self.transactions_table, len);
            }
            Screen::Protocols => match &self.protocol_level {
                ProtocolLevel::List => {
                    let len = self.protocols.len();
                    scroll_table_down(&mut self.protocol_list_state, len);
                }
                ProtocolLevel::Detail { protocol_index } => {
                    let len = self
                        .protocols
                        .get(*protocol_index)
                        .map(|p| p.chains.iter().map(|c| c.positions.len()).sum::<usize>())
                        .unwrap_or(0);
                    scroll_table_down(&mut self.protocol_detail_state, len);
                }
                ProtocolLevel::Assets {
                    protocol_index,
                    chain_index,
                    position_index,
                } => {
                    let len = self
                        .protocols
                        .get(*protocol_index)
                        .and_then(|p| p.chains.get(*chain_index))
                        .and_then(|c| c.positions.get(*position_index))
                        .map(|p| p.assets.len())
                        .unwrap_or(0);
                    scroll_table_down(&mut self.protocol_assets_state, len);
                }
            },
            _ => {}
        }
    }

    pub fn scroll_up(&mut self) {
        match self.active_screen {
            Screen::Holdings => scroll_table_up(&mut self.holdings_table),
            Screen::Transactions => scroll_table_up(&mut self.transactions_table),
            Screen::Protocols => match &self.protocol_level {
                ProtocolLevel::List => scroll_table_up(&mut self.protocol_list_state),
                ProtocolLevel::Detail { .. } => scroll_table_up(&mut self.protocol_detail_state),
                ProtocolLevel::Assets { .. } => scroll_table_up(&mut self.protocol_assets_state),
            },
            _ => {}
        }
    }

    pub fn jump_top(&mut self) {
        match self.active_screen {
            Screen::Holdings if !self.holdings.is_empty() => {
                self.holdings_table.select(Some(0));
            }
            Screen::Transactions if !self.transactions.is_empty() => {
                self.transactions_table.select(Some(0));
            }
            Screen::Protocols => match &self.protocol_level {
                ProtocolLevel::List if !self.protocols.is_empty() => {
                    self.protocol_list_state.select(Some(0));
                }
                ProtocolLevel::Detail { .. } => {
                    self.protocol_detail_state.select(Some(0));
                }
                ProtocolLevel::Assets { .. } => {
                    self.protocol_assets_state.select(Some(0));
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn jump_bottom(&mut self) {
        match self.active_screen {
            Screen::Holdings if !self.holdings.is_empty() => {
                self.holdings_table.select(Some(self.holdings.len() - 1));
            }
            Screen::Transactions if !self.transactions.is_empty() => {
                self.transactions_table
                    .select(Some(self.transactions.len() - 1));
            }
            Screen::Protocols => match &self.protocol_level {
                ProtocolLevel::List if !self.protocols.is_empty() => {
                    self.protocol_list_state
                        .select(Some(self.protocols.len() - 1));
                }
                ProtocolLevel::Detail { protocol_index } => {
                    if let Some(proto) = self.protocols.get(*protocol_index) {
                        let len: usize =
                            proto.chains.iter().map(|c| c.positions.len()).sum();
                        if len > 0 {
                            self.protocol_detail_state.select(Some(len - 1));
                        }
                    }
                }
                ProtocolLevel::Assets {
                    protocol_index,
                    chain_index,
                    position_index,
                } => {
                    if let Some(pos) = self
                        .protocols
                        .get(*protocol_index)
                        .and_then(|p| p.chains.get(*chain_index))
                        .and_then(|c| c.positions.get(*position_index))
                    {
                        if !pos.assets.is_empty() {
                            self.protocol_assets_state
                                .select(Some(pos.assets.len() - 1));
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn enter(&mut self) {
        if self.active_screen != Screen::Protocols {
            return;
        }

        match self.protocol_level.clone() {
            ProtocolLevel::List => {
                if let Some(idx) = self.protocol_list_state.selected() {
                    if idx < self.protocols.len() {
                        self.protocol_level = ProtocolLevel::Detail {
                            protocol_index: idx,
                        };
                        self.protocol_detail_state.select(Some(0));
                    }
                }
            }
            ProtocolLevel::Detail { protocol_index } => {
                if let Some(selected) = self.protocol_detail_state.selected() {
                    if let Some(proto) = self.protocols.get(protocol_index) {
                        let mut idx = 0;
                        for (ci, chain) in proto.chains.iter().enumerate() {
                            for (pi, _) in chain.positions.iter().enumerate() {
                                if idx == selected {
                                    self.protocol_level = ProtocolLevel::Assets {
                                        protocol_index,
                                        chain_index: ci,
                                        position_index: pi,
                                    };
                                    self.protocol_assets_state.select(Some(0));
                                    return;
                                }
                                idx += 1;
                            }
                        }
                    }
                }
            }
            ProtocolLevel::Assets { .. } => {}
        }
    }

    pub fn go_back(&mut self) {
        if self.active_screen != Screen::Protocols {
            return;
        }

        match self.protocol_level.clone() {
            ProtocolLevel::List => {}
            ProtocolLevel::Detail { .. } => {
                self.protocol_level = ProtocolLevel::List;
            }
            ProtocolLevel::Assets {
                protocol_index, ..
            } => {
                self.protocol_level = ProtocolLevel::Detail { protocol_index };
            }
        }
    }
}

fn scroll_table_down(state: &mut TableState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => {
            if i >= len - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

fn scroll_table_up(state: &mut TableState) {
    let i = match state.selected() {
        Some(0) | None => 0,
        Some(i) => i - 1,
    };
    state.select(Some(i));
}

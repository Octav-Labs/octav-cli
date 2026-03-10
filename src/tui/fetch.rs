use std::sync::mpsc;

use crate::client::OctavClient;
use crate::tui::data;
use crate::tui::event::DataEvent;

pub fn spawn_fetch(api_key: String, addresses: Vec<String>, tx: mpsc::Sender<DataEvent>) {
    std::thread::spawn(move || {
        let _ = tx.send(DataEvent::LoadingStarted);

        let client = OctavClient::new(api_key);

        // Credits (free)
        match client.get_credits() {
            Ok(json) => {
                let credits = data::parse_credits(&json);
                let _ = tx.send(DataEvent::CreditsLoaded(credits));
            }
            Err(e) => {
                let _ = tx.send(DataEvent::FetchError(format!("Credits: {}", e)));
            }
        }

        // NAV (1 credit/addr)
        match client.get_nav(&addresses, "USD") {
            Ok(json) => {
                let nav = data::parse_nav(&json);
                let _ = tx.send(DataEvent::NavLoaded(nav));
            }
            Err(e) => {
                let _ = tx.send(DataEvent::FetchError(format!("NAV: {}", e)));
            }
        }

        // Portfolio (1 credit/addr)
        match client.get_portfolio(&addresses) {
            Ok(json) => {
                let (protocols, holdings, chains) = data::parse_portfolio(&json);
                let _ = tx.send(DataEvent::PortfolioLoaded {
                    protocols,
                    holdings,
                    chains,
                });
            }
            Err(e) => {
                let _ = tx.send(DataEvent::FetchError(format!("Portfolio: {}", e)));
            }
        }

        // Transactions (1 credit/addr)
        match client.get_transactions(&addresses, None, None, None, None, 0, 250) {
            Ok(json) => {
                let (items, total) = data::parse_transactions(&json);
                let _ = tx.send(DataEvent::TransactionsLoaded { items, total });
            }
            Err(e) => {
                let _ = tx.send(DataEvent::FetchError(format!("Transactions: {}", e)));
            }
        }
    });
}

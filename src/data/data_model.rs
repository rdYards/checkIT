use crate::data::ledger::Ledger;
use crate::data::ledger_db::{LedgerBannerInfo, LedgerDatabase, LockEvent};
use std::sync::Arc;
use tokio::sync::watch;

/// A snapshot of ledger data relevant to the UI.
#[derive(Clone, Debug)]
pub struct UiLedger {
    pub banner: LedgerBannerInfo,
    pub data: Option<Ledger>,
}

/// The reactive data model for the UI.
/// Holds a snapshot of ledger data and notifies subscribers of changes.
#[derive(Clone)]
pub struct DataModel {
    /// Receiver for UI ledgers. Use this to update the UI.
    pub ledgers: watch::Receiver<Vec<UiLedger>>,
    ledgers_tx: watch::Sender<Vec<UiLedger>>,
}

impl DataModel {
    /// Creates a new `DataModel` and subscribes it to the `LedgerDatabase`.
    pub fn new(db: Arc<LedgerDatabase>) -> Self {
        let (tx, rx) = watch::channel(Vec::new());
        let model = Self {
            ledgers: rx,
            ledgers_tx: tx,
        };
        model.subscribe_to_db(db);
        model
    }

    /// Subscribes to database events and updates the UI ledgers.
    fn subscribe_to_db(&self, db: Arc<LedgerDatabase>) {
        let mut receiver = db
            .subscribe_lock_events()
            .expect("Failed to subscribe to events");
        let tx = self.ledgers_tx.clone();

        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                let mut ledgers = tx.borrow().clone();
                match event {
                    LockEvent::LedgerAdded(key) => {
                        if let Some(info) = db.get_ledger_info(&key) {
                            if let Some(ledger) = db.get_ledger_data(&key) {
                                ledgers.push(UiLedger {
                                    banner: info,
                                    data: Some(ledger),
                                });
                            }
                        }
                    }
                    LockEvent::LedgerRemoved(key) => {
                        ledgers.retain(|l| l.banner.key != key);
                    }
                    LockEvent::LedgerUpdated(key) => {
                        if let Some(info) = db.get_ledger_info(&key) {
                            if let Some(ledger) = db.get_ledger_data(&key) {
                                if let Some(ledger_ui) =
                                    ledgers.iter_mut().find(|l| l.banner.key == key)
                                {
                                    ledger_ui.banner = info;
                                    ledger_ui.data = Some(ledger);
                                }
                            }
                        }
                    }
                }
                // Notify subscribers (UI) of the change
                let _ = tx.send(ledgers);
            }
        });
    }
}

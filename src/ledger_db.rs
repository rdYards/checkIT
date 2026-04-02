use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::ledger::{Ledger, LedgerState};

#[derive(Debug, Clone)]
pub enum LockEvent {
    LockAcquired(String),
    LockReleased(String),
    LedgerAdded(String),
    LedgerRemoved(String),
}

#[derive(Debug, Clone)]
pub struct LedgerBannerInfo {
    pub key: String,
    pub title: String,
    pub state: LedgerState,
}

// Database struct
#[derive(Default)]
pub struct LedgerDatabase {
    ledgers: Arc<RwLock<HashMap<String, Ledger>>>,
    // For lock event subscriptions
    lock_events: Arc<RwLock<Vec<std::sync::mpsc::Sender<LockEvent>>>>,
}
impl LedgerDatabase {
    pub fn new() -> Self {
        Self {
            ledgers: Arc::new(RwLock::new(HashMap::new())),
            lock_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_ledger(&self, key: String, ledger: Ledger) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_err(|e| e.to_string())?;
        ledgers.insert(key.clone(), ledger);

        // Notify subscribers of ledger addition
        let events = self.lock_events.read().map_err(|e| e.to_string())?;
        for tx in events.iter() {
            let _ = tx.send(LockEvent::LedgerAdded(key.clone()));
        }

        Ok(())
    }

    pub fn remove_ledger(&self, key: &str) -> Result<Option<Ledger>, String> {
        let mut ledgers = self.ledgers.write().map_err(|e| e.to_string())?;
        let ledger = ledgers.remove(key);

        // Notify subscribers of ledger removal
        let events = self.lock_events.read().map_err(|e| e.to_string())?;
        for tx in events.iter() {
            let _ = tx.send(LockEvent::LedgerRemoved(key.to_string()));
        }

        Ok(ledger)
    }

    pub fn request_ledger(
        &self,
        key: &str,
        requester: &str,
        is_user: bool,
    ) -> Result<Option<Ledger>, String> {
        let mut ledgers = self.ledgers.write().map_err(|e| e.to_string())?;

        if let Some(ledger) = ledgers.get_mut(key) {
            // Check current state
            match &ledger.state {
                LedgerState::Unlocked => {
                    // Lock the ledger for the requester
                    if is_user {
                        ledger.state = LedgerState::UserLocked(requester.to_string());
                    } else {
                        ledger.state = LedgerState::SystemLocked(requester.to_string());
                    }

                    // Notify subscribers of state change
                    let events = self.lock_events.read().map_err(|e| e.to_string())?;
                    for tx in events.iter() {
                        let _ = tx.send(LockEvent::LockAcquired(key.to_string()));
                    }

                    // Return the ledger
                    Ok(Some(ledger.clone()))
                }
                LedgerState::UserLocked(current_user) if is_user => {
                    // If it's a user requesting and another user has it locked,
                    // automatically unlock it for the new user
                    ledger.state = LedgerState::UserLocked(requester.to_string());

                    // Notify subscribers of state change
                    let events = self.lock_events.read().map_err(|e| e.to_string())?;
                    for tx in events.iter() {
                        let _ = tx.send(LockEvent::LockAcquired(key.to_string()));
                    }

                    Ok(Some(ledger.clone()))
                }
                LedgerState::SystemLocked(_) if is_user => {
                    // If a user requests a system-locked ledger, unlock it
                    ledger.state = LedgerState::UserLocked(requester.to_string());

                    // Notify subscribers of state change
                    let events = self.lock_events.read().map_err(|e| e.to_string())?;
                    for tx in events.iter() {
                        let _ = tx.send(LockEvent::LockAcquired(key.to_string()));
                    }

                    Ok(Some(ledger.clone()))
                }
                _ => {
                    // Ledger is locked by another process
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn contains_ledger(&self, key: &str) -> Result<bool, String> {
        let ledgers = self.ledgers.read().map_err(|e| e.to_string())?;
        Ok(ledgers.contains_key(key))
    }

    pub fn contains_any_ledgers(&self) -> Result<bool, String> {
        let ledgers = self.ledgers.read().map_err(|e| e.to_string())?;
        Ok(!ledgers.is_empty())
    }

    pub fn subscribe_lock_events(&self) -> Result<std::sync::mpsc::Receiver<LockEvent>, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut subscribers = self.lock_events.write().map_err(|e| e.to_string())?;
        subscribers.push(tx);
        Ok(rx)
    }

    pub fn get_banner_info(&self, key: &str) -> Option<LedgerBannerInfo> {
        let ledgers = self.ledgers.read().map_err(|e| e.to_string()).ok()?;
        ledgers.get(key).map(|ledger| LedgerBannerInfo {
            key: key.to_string(), // Using the database key instead
            title: ledger.data.meta.title.clone(),
            state: ledger.state.clone(),
        })
    }
}
impl Drop for LedgerDatabase {
    fn drop(&mut self) {
        // Clean up resources here
        // Since there are no external resources to clean up (like file handles),
        // we can just let Rust handle the cleanup automatically
    }
}

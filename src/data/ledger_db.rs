use crate::data::ledger::{Ledger, LedgerState};
use std::{
    collections::HashMap,
    sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// Events emitted by the LedgerDatabase.
#[derive(Debug, Clone)]
pub enum LockEvent {
    LedgerAdded(String),
    LedgerRemoved(String),
    LedgerUpdated(String),
}

/// Information about a ledger, used for UI display.
#[derive(Debug, Clone)]
pub struct LedgerBannerInfo {
    pub key: String,
    pub title: String,
    pub state: LedgerState, // Unused at the moment will be used for future backgroun processing
}

/// The thread-safe ledger database.
/// Manages all active ledgers and notifies subscribers of changes.
#[derive(Default)]
pub struct LedgerDatabase {
    ledgers: Arc<RwLock<HashMap<String, Ledger>>>,
    pub lock_events: Arc<RwLock<Vec<UnboundedSender<LockEvent>>>>,
}

impl LedgerDatabase {
    /// Creates a new, empty LedgerDatabase.
    pub fn new() -> Self {
        Self {
            ledgers: Arc::new(RwLock::new(HashMap::new())),
            lock_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate a short random/unique string to append to ledger keys
    /// Prevents collisions when importing clones or ledgers with the same name
    fn generate_unique_id(&self) -> String {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        // Use the last 16 characters of the nanosecond timestamp as unique suffix
        format!("{:x}", start).chars().rev().take(16).collect()
    }

    /// Emits a `LockEvent` to all subscribers.
    fn emit(&self, event: LockEvent) {
        if let Ok(events) = self.lock_events.read() {
            for tx in events.iter() {
                let _ = tx.send(event.clone());
            }
        }
    }

    /// Creates a new ledger and adds it to the database.
    pub fn create_ledger(
        &self,
        title: String,
        description: String,
        password: String,
    ) -> Result<(), String> {
        let key = format!("{}_{}", title, self.generate_unique_id());
        let ledger = Ledger::new(&password, &title, &description);
        self.add_ledger_internal(key, ledger)?;
        Ok(())
    }

    /// Imports an existing ledger into the database.
    pub fn import_ledger(&self, path: String, password: String) -> Result<(), String> {
        let mut ledger = Ledger::from_file(&password, &path)
            .ok_or_else(|| "Invalid password or file not found".to_string())?;
        
        // Update root path to prevent saving to the wrong location
        ledger.data.meta.root_path = std::path::PathBuf::from(&path);
        
        let key = format!("{}_{}", path, self.generate_unique_id());
        self.add_ledger_internal(key, ledger)?;
        Ok(())
    }

    /// Clone an existing ledger and adds it to the database
    pub fn clone_ledger(
        &self,
        old_key: &str,
        new_title: String,
        new_password: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch existing data using old_key
        let original_ledger = self
            .get_ledger_data(old_key)
            .ok_or("Original ledger not found")?;

        let new_key = format!("{}_{}", new_title, self.generate_unique_id());
        let description = original_ledger.data.meta.description.clone();

        // Create new Ledger instance with new password and title
        let mut new_ledger = Ledger::new(&new_password, &new_title, &description);

        // Copy the entries from the old ledger to the new one
        for entry in &original_ledger.data.ledger {
            new_ledger
                .data
                .create_entry(entry.genre.clone(), entry.data.clone())
                .map_err(|e| e.to_string())?;
        }

        self.add_ledger_internal(new_key, new_ledger)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Takes ledger already saved and overrides previous .sl file
    pub fn save_ledger_to_disk(&self, key: &str, password: String) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_err(|e| e.to_string())?;
        if let Some(ledger) = ledgers.get_mut(key) {
            ledger.save(&password).map_err(|e| e.to_string())?;
            // Emit an update event for Data Model
            self.emit(LockEvent::LedgerUpdated(key.to_string()));
            Ok(())
        } else {
            Err("Ledger not found".to_string())
        }
    }

    /// Saves ledger to a location requested by user
    pub fn save_ledger_as(&self, key: &str, path: &str, password: String) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_err(|e| e.to_string())?;
        if let Some(ledger) = ledgers.get_mut(key) {
            // Update the root path before saving
            ledger.data.meta.root_path = std::path::PathBuf::from(path);
            ledger.save(&password).map_err(|e| e.to_string())?;
            self.emit(LockEvent::LedgerUpdated(key.to_string()));
            Ok(())
        } else {
            Err("Ledger not found".to_string())
        }
    }

    /// Adds a ledger to the database and emits an event.
    fn add_ledger_internal(&self, key: String, ledger: Ledger) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_rel_err()?;
        ledgers.insert(key.clone(), ledger);
        self.emit(LockEvent::LedgerAdded(key));
        Ok(())
    }

    /// Removes a ledger from the database and emits an event.
    pub fn remove_ledger(&self, key: &str) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_rel_err()?;
        if ledgers.remove(key).is_some() {
            self.emit(LockEvent::LedgerRemoved(key.to_string()));
        }
        Ok(())
    }

    /// Adds an entry to a ledger and emits an event.
    pub fn add_entry_to_ledger(
        &self,
        key: String,
        genre: String,
        data: String,
    ) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_rel_err()?;
        if let Some(ledger) = ledgers.get_mut(&key) {
            ledger
                .data
                .create_entry(genre, data)
                .map_err(|e| e.to_string())?;

            println!("[DEBUG] About to emit LedgerUpdated event for key: {}", key);

            self.emit(LockEvent::LedgerUpdated(key.clone()));

            println!(
                "[DEBUG] Emitted LedgerUpdated event for key: {}",
                key.clone()
            );
            Ok(())
        } else {
            Err("Ledger not found".to_string())
        }
    }

    /// Removes an entry from a ledger and emits an event.
    pub fn remove_entry_from_ledger(&self, key: String, entry_id: String) -> Result<(), String> {
        let mut ledgers = self.ledgers.write().map_rel_err()?;
        if let Some(ledger) = ledgers.get_mut(&key) {
            ledger
                .data
                .remove_entry(&entry_id)
                .map_err(|e| e.to_string())?;
            self.emit(LockEvent::LedgerUpdated(key));
            Ok(())
        } else {
            Err("Ledger not found".to_string())
        }
    }

    /// Gets all ledger keys.
    /// Gets ledger information for display in the UI.
    pub fn get_ledger_info(&self, key: &str) -> Option<LedgerBannerInfo> {
        let ledgers = self.ledgers.read().ok()?;
        ledgers.get(key).map(|ledger| LedgerBannerInfo {
            key: key.to_string(),
            title: ledger.data.meta.title.clone(),
            state: ledger.state.clone(),
        })
    }

    /// Gets the full ledger data.
    pub fn get_ledger_data(&self, key: &str) -> Option<Ledger> {
        self.ledgers.read().ok()?.get(key).cloned()
    }

    /// Subscribes to ledger change events.
    pub fn subscribe_lock_events(&self) -> Result<UnboundedReceiver<LockEvent>, String> {
        let (tx, rx) = unbounded_channel();
        let mut subscribers = self.lock_events.write().map_err(|e| e.to_string())?;
        subscribers.push(tx);
        Ok(rx)
    }
}

// Helper trait to reduce boilerplate for RwLock error handling.
trait MapErrExt<T> {
    fn map_rel_err(self) -> Result<T, String>;
}

impl<T> MapErrExt<T> for Result<T, PoisonError<RwLockReadGuard<'_, HashMap<String, Ledger>>>> {
    fn map_rel_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

impl<T> MapErrExt<T> for Result<T, PoisonError<RwLockWriteGuard<'_, HashMap<String, Ledger>>>> {
    fn map_rel_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

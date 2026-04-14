use crate::DEFAULT_FILE_PATH;
use sl::SecureLedger;
use std::path::Path;

// Database states for ledgers
// Used to prevent processes from writing to ledgers at the same time
#[derive(Debug, Clone, PartialEq)]
pub enum LedgerState {
    Unlocked,
    UserLocked(String),   // Username who has it locked
    SystemLocked(String), // System process that has it locked
}

#[derive(Clone, Debug)]
pub struct Ledger {
    pub data: SecureLedger,
    pub state: LedgerState,
}

impl Ledger {
    pub fn new(password: &str, title: &str, discription: &str) -> Self {
        let mut ledger = SecureLedger::initialize(None, Some(password)).unwrap();
        let _ = ledger.update_meta(DEFAULT_FILE_PATH, title, discription);
        Self {
            data: ledger,
            state: LedgerState::Unlocked,
        }
    }

    pub fn from_file(password: &str, file_path: &str) -> Option<Self> {
        if Path::new(file_path).exists() {
            match SecureLedger::initialize(Some(file_path), Some(password)) {
                Ok(ledger) => Some(Self {
                    data: ledger,
                    state: LedgerState::Unlocked,
                }),
                Err(e) => {
                    eprintln!("Error loading ledger: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    pub async fn save(&mut self, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.data.upload_to_sl(password).ok();
        Ok(())
    }
}

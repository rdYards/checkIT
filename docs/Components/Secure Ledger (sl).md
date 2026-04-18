CheckIT uses the `sl` crate as its core cryptographic engine for all data persistence and security.

## Core Functionality

The application wraps `sl::SecureLedger` within a `Ledger` struct to manage both the encrypted data and the `Led/Unlocked` state.

### Data Encryption
- **AES-256-GCM**: Every ledger is encrypted using AES-256-GCM.
- **Argon2**: Passwords provided via `AlertDialog` are processed through Argon2 to derive strong encryption keys.
- **Integrity**: The `sl` crate ensures that any tampering with the `.sl` file is detected via cryptographic hashing.

### Data Structure
The following structures from `sl` are utilized within the `Ledger` implementation:

#### `LedgerEntry`
The fundamental unit of data within a ledger.
- `id`: A unique identifier for the entry.
- `genre`: A category tag (e.g., "commit", "push").
- `data`: The sensitive payload.
- `timestamp`: When the entry was created.

#### `MetaData`
Stored within the encrypted archive to provide context without requiring decryption.
- `title`: The display name of the ledger.
- `description`: A summary of the ledger's purpose.
- `root_path`: The filesystem path where the ledger is anchored.
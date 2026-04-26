### What is CheckIT?
CheckIT is a secure, encrypted personal ledger application designed to help you maintain organized, private, and tamper-proof records using industry-standard encryption.

### What platforms does CheckIT support?
CheckIT currently supports:
- Linux (Debian/Ubuntu, Fedora/RHEL/CentOS and Flatpak)
- macOS (10.13 and later)
- Windows (in development)

### Is CheckIT open source?
Yes, CheckIT is fully open source under the GPLv2.0 provided in the repository.

### How secure is my data?
CheckIT uses AES-256-GCM encryption with Argon2 key derivation. Each ledger is encrypted with a unique key derived from your password, ensuring your data remains secure even if the application files are compromised.
## Installation & Setup

### Why does the macOS app show a "damaged" warning?
This is a Gatekeeper warning. CheckIT is not part of the MacOS Developer Program, so the app is not code-signed. You can bypass this by:
1. Opening System Settings
2. Going to Privacy & Security
3. Scrolling down to the "Open Anyway" button for CheckIT
### Can I install CheckIT using the package managers?
No, at the current moment CheckIT is not apart of any package managers.

### Can I install CheckIT without using the package managers?
Yes! You can build from source by following the instructions in the development documentation.

### What are the system requirements for CheckIT?
- Linux: GTK4, Libadwaita
- macOS: GTK4, Libadwaita (via Homebrew)

### Why can't I find CheckIT in my application menu after installation?
If you installed via Flatpak or built from source, you may need to manually create a desktop entry file. See the installation documentation for details.

## Usage Questions

### How do I recover my password if I forget it?
Unfortunately, there is no "Forgot Password" feature. If you lose your password, your data is lost forever. Always keep a secure backup of your passwords.

### Can I share ledgers between different devices?
Yes! You can export ledgers and import them on other devices. For local network sharing, CheckIT supports peer-to-peer sharing between instances on the same LAN.

### What's the difference between Save and Save As?
- **Save**: Saves changes back to the original ledger file
- **Save As**: Creates a new copy of the ledger with a different name/location

### Can I search across multiple ledgers?
Currently, you can only search within a single ledger. Searching across multiple ledgers is planned for future updates.

### Why can't I see my entries after importing a ledger?
If your entries aren't appearing:
1. Verify you entered the correct password
2. Check if the ledger is properly unlocked (should show "Unlocked" status)
3. Try restarting the application

## Technical Questions

### What encryption does CheckIT use?
CheckIT uses AES-256-GCM for file encryption with Argon2 key derivation. Peer-to-peer sharing uses X25519 for key exchange.

### Can I trust the peer-to-peer sharing feature?
Yes. When sharing with peers, the data is encrypted specifically for the recipient using their public key, ensuring only they can decrypt it.

### What file format do ledgers use?
Ledgers are stored in a custom `.sl` format that contains the encrypted data along with metadata.

### Why does the application build fail on my system?
Common build issues:
- Missing GTK4/Libadwaita development packages
- Outdated Rust toolchain (update with `rustup update`)
- Permission issues when installing cargo plugins

## Future Features

### Will there be auto-save functionality?
Auto-save is planned for future updates. Currently, you need to manually save changes.

### Will Windows support be added soon?
Windows support is in development but currently commented out in the release workflow. It will be available in future releases.

## Troubleshooting

### The application crashes when I try to load a ledger
This could be due to:
- Corrupted ledger file
- Incompatible version of the ledger format
- Missing dependencies

### Entries appear scrambled or unreadable
This usually indicates file corruption. Try:
1. Making a backup of the ledger file
2. Attempting to import it again
3. If the issue persists, the file may be unrecoverable

### Keyboard shortcuts aren't working
Check if:
- The application has focus
- Your keyboard layout supports the specified shortcuts
- The shortcuts haven't been overridden by your window manager

### The app looks blurry or has display issues
This is often related to scaling. Try:
- Adjusting your system display scaling settings
- Running with `GTK_SCALE=2` environment variable for 2x scaling

If you have additional questions not covered here, please check the documentation or open an issue in the GitHub repository.

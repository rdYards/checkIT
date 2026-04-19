use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use gethostname::gethostname;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use sl::SecureLedger;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{RwLock, mpsc},
};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::data::ledger_db::LedgerDatabase;

pub const DISCOVERY_PORT: u16 = 53333;
pub const TRANSFER_PORT: u16 = 53334;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2PMessage {
    Discovery {
        public_key: [u8; 32],
        device_name: String,
    },
    TransferRequest {
        sender_name: String,
        data_type: TransferType,
        payload_size: u64,
        sender_pubkey: [u8; 32],
    },
    TransferResponse {
        accepted: bool,
    },
    Payload {
        encrypted_data: Vec<u8>,
        nonce: [u8; 12],
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransferType {
    FullLedger,
    SingleEntry,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub pubkey: PublicKey,
    pub addr: SocketAddr,
    pub name: String,
}

pub struct P2PManager {
    local_id: String,
    secret: StaticSecret,
    public: PublicKey,
    db: Arc<LedgerDatabase>,
    request_tx: mpsc::UnboundedSender<IncomingTransfer>,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
}

#[derive(Debug)]
pub struct IncomingTransfer {
    pub sender_name: String,
    pub data_type: TransferType,
    pub sender_pubkey: PublicKey,
    pub stream: TcpStream,
}

impl P2PManager {
    pub fn new(
        db: Arc<LedgerDatabase>,
        request_tx: mpsc::UnboundedSender<IncomingTransfer>,
    ) -> Self {
        // Seed is randomly generated upon start-up as it's
        // only needed for network opperations
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).expect("Failed to generate random seed");

        let secret = StaticSecret::from(seed);
        let public = PublicKey::from(&secret);

        Self {
            local_id: gethostname()
                .into_string()
                .expect("Failed to convert hostname to String"),
            secret,
            public,
            db,
            request_tx,
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Starts all tokio thread for network sharing
    pub async fn start(self: Arc<Self>) {
        let manager_discovery = self.clone();
        let manager_transfer = self.clone();

        let value = manager_discovery.clone();
        tokio::spawn(async move {
            value.listen_for_discovery().await;
        });

        tokio::spawn(async move {
            manager_discovery.broadcast_presence().await;
        });

        tokio::spawn(async move {
            manager_transfer.listen_for_transfers().await;
        });
    }

    /// Sends out a Broadcast on 255.255.255.255 to all networks the device is connected to
    /// Does not support IPv6
    async fn broadcast_presence(&self) {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        socket.set_broadcast(true).unwrap();

        let msg = P2PMessage::Discovery {
            public_key: self.public.to_bytes(),
            device_name: self.local_id.clone(),
        };

        let encoded = postcard::to_allocvec(&msg).expect("Failed to serialize discovery");
        let target = format!("255.255.255.255:{}", DISCOVERY_PORT);

        loop {
            let _ = socket.send_to(&encoded, &target).await;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    /// Returns a snapshot of currently discovered peers
    pub async fn get_discovered_peers(&self) -> Vec<(String, PeerInfo)> {
        let peers = self.peers.read().await;
        peers
            .iter()
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Checks on the designated port for any other clients
    /// If found will add to peers. Does filter out its own Broadcasts out
    async fn listen_for_discovery(&self) {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT))
            .await
            .unwrap();
        let mut buf = [0u8; 1024];
        let mut last_update = std::time::Instant::now();
        let update_interval = std::time::Duration::from_secs(5);

        loop {
            if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                // Skip messages from our own IP address
                if addr.ip() == local_ip().expect("Failed to get local IP") {
                    continue;
                }

                if let Ok(P2PMessage::Discovery {
                    public_key,
                    device_name,
                }) = postcard::from_bytes(&buf[..len])
                {
                    // Only update the peer list if enough time has passed
                    if last_update.elapsed() >= update_interval {
                        let mut peers = self.peers.write().await;
                        peers.insert(
                            device_name.clone(),
                            PeerInfo {
                                pubkey: PublicKey::from(public_key),
                                addr,
                                name: device_name,
                            },
                        );
                        last_update = std::time::Instant::now();
                    }
                }
            }
        }
    }

    /// Checks for user Broadcasts and returns with client information
    async fn listen_for_transfers(&self) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", TRANSFER_PORT))
            .await
            .unwrap();

        loop {
            if let Ok((mut stream, _addr)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                if let Ok(len) = stream.read(&mut buf).await {
                    if let Ok(P2PMessage::TransferRequest {
                        sender_name,
                        data_type,
                        sender_pubkey,
                        ..
                    }) = postcard::from_bytes(&buf[..len])
                    {
                        let _ = self.request_tx.send(IncomingTransfer {
                            sender_name,
                            data_type,
                            sender_pubkey: PublicKey::from(sender_pubkey),
                            stream,
                        });
                    }
                }
            }
        }
    }

    /// Encrypts data using X25519 Diffie-Hellman and AES-256-GCM
    fn encrypt_payload(&self, remote_pub: PublicKey, plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
        let shared_secret = self.secret.diffie_hellman(&remote_pub);
        let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_bytes());
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        getrandom::fill(&mut nonce_bytes).expect("Failed to generate nonce");
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .expect("Encryption failure");
        (ciphertext, nonce_bytes)
    }

    /// Decrypts data received from a peer
    fn decrypt_payload(
        &self,
        remote_pub: PublicKey,
        ciphertext: &[u8],
        nonce_bytes: &[u8; 12],
    ) -> Result<Vec<u8>, String> {
        let shared_secret = self.secret.diffie_hellman(&remote_pub);
        let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_bytes());
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
    }

    /// Sends encrypted Ledger over the network to peer
    pub async fn send_ledger(&self, target_name: String, ledger_key: String) -> Result<(), String> {
        // Resolve Peer
        let peer = {
            let peers = self.peers.read().await;
            peers
                .get(&target_name)
                .cloned()
                .ok_or("Peer not found in registry")?
        };

        // Prepare Ledger Data
        let ledger = self
            .db
            .get_ledger_data(&ledger_key)
            .ok_or("Ledger not found")?;

        // Encrypt data to stream over network
        let raw_data = postcard::to_allocvec(&ledger.data)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let (encrypted_data, nonce) = self.encrypt_payload(peer.pubkey, &raw_data);

        // Connect and Handshake
        let mut stream = TcpStream::connect(format!("{}:{}", peer.addr.ip(), TRANSFER_PORT))
            .await
            .map_err(|e| e.to_string())?;

        let req = P2PMessage::TransferRequest {
            sender_name: self.local_id.clone(),
            data_type: TransferType::FullLedger,
            payload_size: encrypted_data.len() as u64,
            sender_pubkey: self.public.to_bytes(),
        };

        stream
            .write_all(&postcard::to_allocvec(&req).unwrap())
            .await
            .map_err(|e| e.to_string())?;

        // Wait for TransferResponse
        let mut resp_buf = [0u8; 128];
        let len = stream
            .read(&mut resp_buf)
            .await
            .map_err(|e| e.to_string())?;

        let response: P2PMessage = postcard::from_bytes(&resp_buf[..len])
            .map_err(|e| format!("Failed to deserialize response: {}", e))?;

        let P2PMessage::TransferResponse { accepted } = response else {
            return Err("Received unexpected message type".to_string());
        };

        if !accepted {
            return Err("Transfer rejected by peer".to_string());
        }

        // Only now send the payload
        let payload_msg = P2PMessage::Payload {
            encrypted_data,
            nonce,
        };
        stream
            .write_all(&postcard::to_allocvec(&payload_msg).unwrap())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Sends an encrypted single entry from a specific ledger to a peer
    pub async fn send_entry(
        &self,
        target_name: String,
        ledger_key: String,
        entry_id: String,
    ) -> Result<(), String> {
        let peer = {
            let peers = self.peers.read().await;
            peers.get(&target_name).cloned().ok_or("Peer not found")?
        };

        let ledger = self
            .db
            .get_ledger_data(&ledger_key)
            .ok_or("Ledger not found")?;
        let entry = ledger
            .data
            .ledger
            .iter()
            .find(|e| e.id == entry_id)
            .ok_or("Entry not found")?;

        // Create a tuple with the entry components instead of serializing the whole LedgerEntry
        let entry_components = (entry.genre.clone(), entry.data.clone());
        let raw_data = postcard::to_allocvec(&entry_components).map_err(|e| e.to_string())?;
        let (encrypted_data, nonce) = self.encrypt_payload(peer.pubkey, &raw_data);

        let mut stream = TcpStream::connect(format!("{}:{}", peer.addr.ip(), TRANSFER_PORT))
            .await
            .map_err(|e| e.to_string())?;

        let req = P2PMessage::TransferRequest {
            sender_name: self.local_id.clone(),
            data_type: TransferType::SingleEntry,
            payload_size: encrypted_data.len() as u64,
            sender_pubkey: self.public.to_bytes(),
        };

        stream
            .write_all(&postcard::to_allocvec(&req).unwrap())
            .await
            .map_err(|e| e.to_string())?;

        // Wait for TransferResponse
        let mut resp_buf = [0u8; 128];
        let len = stream
            .read(&mut resp_buf)
            .await
            .map_err(|e| e.to_string())?;

        let response: P2PMessage = postcard::from_bytes(&resp_buf[..len])
            .map_err(|e| format!("Failed to deserialize response: {}", e))?;

        // Catch all !Accepted to prevent issues
        let P2PMessage::TransferResponse { accepted } = response else {
            return Err("Received unexpected message type".to_string());
        };

        if !accepted {
            return Err("Transfer rejected by peer".to_string());
        }

        let payload_msg = P2PMessage::Payload {
            encrypted_data,
            nonce,
        };
        stream
            .write_all(&postcard::to_allocvec(&payload_msg).unwrap())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Helper for the UI thread to finalize the receipt of data
    pub async fn handle_incoming_stream(
        &self,
        mut stream: TcpStream,
        sender_pubkey: PublicKey,
        data_type: TransferType,
    ) -> Result<Vec<u8>, String> {
        // Tell sender we accept
        let resp = P2PMessage::TransferResponse { accepted: true };
        stream
            .write_all(&postcard::to_allocvec(&resp).unwrap())
            .await
            .map_err(|e| e.to_string())?;

        // Read payload
        let mut buf = Vec::new();
        stream
            .read_to_end(&mut buf)
            .await
            .map_err(|e| e.to_string())?;

        if let Ok(P2PMessage::Payload {
            encrypted_data,
            nonce,
        }) = postcard::from_bytes(&buf)
        {
            let decrypted = self.decrypt_payload(sender_pubkey, &encrypted_data, &nonce)?;

            // Process the received data
            match data_type {
                TransferType::FullLedger => {
                    // Deserialize and import the full ledger
                    let ledger: SecureLedger = postcard::from_bytes(&decrypted)
                        .map_err(|e| format!("Deserialization error: {}", e))?;

                    self.db
                        .import_ledger_internal(ledger)
                        .map_err(|e| format!("Failed to import received ledger: {}", e))?;
                }
                TransferType::SingleEntry => {
                    // Deserialize the entry components
                    let entry_components: (String, String) = postcard::from_bytes(&decrypted)
                        .map_err(|e| format!("Deserialization error: {}", e))?;

                    // Extract the components
                    let (genre, data) = entry_components;

                    // Add to the current ledger, similar to how the UI does it
                    let current_key = self.db.get_current_ledger_key();
                    if let Some(ledger_key) = current_key.clone().as_ref() {
                        drop(current_key); // Release the read lock
                        self.db
                            .add_entry_to_ledger(ledger_key.clone(), genre, data)
                            .map_err(|e| e.to_string())?;
                    } else {
                        return Err("No current ledger available to add entry to".to_string());
                    }
                }
            }

            Ok(decrypted)
        } else {
            Err("Invalid payload received".to_string())
        }
    }
}

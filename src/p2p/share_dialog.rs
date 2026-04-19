use adw::{
    ActionRow, ApplicationWindow, Dialog,
    prelude::{ActionRowExt, AdwDialogExt, BoxExt, ButtonExt, WidgetExt},
};
use gtk::{Box, Button, Label, ListBox, Orientation};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::p2p::messenger::P2PManager;

/// Enum to handle the "Ledger or LedgerEntry"
#[derive(Clone)]
pub enum ShareTarget {
    FullLedger {
        key: String,
    },
    SingleEntry {
        ledger_key: String,
        entry_id: String,
    },
}

enum ShareResult {
    Success,
    Error(String),
}

pub fn open_share_dialog(
    window: ApplicationWindow,
    p2p_manager: Arc<P2PManager>,
    target: ShareTarget,
) {
    let (tx, mut rx) = mpsc::channel::<ShareResult>(10);

    // Attach receiver to main context using spawn_local
    glib::MainContext::default().spawn_local(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ShareResult::Success => {}   // Add logging later on
                ShareResult::Error(_e) => {} // Add logging later on
            }
        }
    });

    // UI Construction
    let dialog = Dialog::builder().title("Share with Peer").build();
    dialog.set_width_request(600);

    let main_box = Box::new(Orientation::Vertical, 10);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);

    let info_label = Label::new(Some("Select a discovered peer to send data:"));
    main_box.append(&info_label);

    let list_box = ListBox::new();
    list_box.add_css_class("transparent-list");
    main_box.append(&list_box);

    let close_btn = Button::with_label("Cancel");
    close_btn.connect_clicked(glib::clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));
    main_box.append(&close_btn);
    dialog.set_child(Some(&main_box));

    let p2p_clone = p2p_manager.clone();

    // Handles the discovery and display of LAN peers for sharing data.
    glib::MainContext::default().spawn_local(async move {
        let peers = p2p_clone.get_discovered_peers().await;

        if peers.is_empty() {
            let empty_label = Label::new(Some("No peers discovered on LAN..."));
            empty_label.add_css_class("dim-label");
            list_box.append(&empty_label);
        } else {
            for (name, info) in peers {
                let row = ActionRow::builder()
                    .title(&name)
                    .subtitle(&format!("IP: {}", info.addr))
                    .build();

                let send_btn = Button::with_label("Send");
                send_btn.add_css_class("suggested-action");
                send_btn.set_margin_top(5);
                send_btn.set_margin_bottom(5);

                let peer_name = name.clone();
                let p2p_inner = p2p_clone.clone();
                let target_final = target.clone();
                let tx_inner = tx.clone();
                let send_btn_inner = send_btn.clone();

                send_btn.connect_clicked(move |_| {
                    let p2p_final = p2p_inner.clone();
                    let peer_name_final = peer_name.clone();
                    let target_final_inner = target_final.clone();
                    let tx_final = tx_inner.clone();
                    let btn = send_btn_inner.clone();

                    glib::MainContext::default().spawn_local(async move {
                        let result = match target_final_inner {
                            ShareTarget::FullLedger { key } => {
                                p2p_final.send_ledger(peer_name_final, key).await
                            }
                            ShareTarget::SingleEntry {
                                ledger_key,
                                entry_id,
                            } => {
                                p2p_final
                                    .send_entry(peer_name_final, ledger_key, entry_id)
                                    .await
                            }
                        };

                        match result {
                            Ok(_) => {
                                btn.set_label("Received");
                                btn.set_sensitive(false);
                                let _ = tx_final.send(ShareResult::Success).await;
                            }
                            Err(e) => {
                                btn.set_label("Failed");
                                btn.set_sensitive(false);
                                let _ = tx_final.send(ShareResult::Error(e)).await;
                            }
                        }
                    });
                });

                row.add_suffix(&send_btn);
                list_box.append(&row);
            }
        }
        dialog.present(Some(&window.clone()));
    });
}

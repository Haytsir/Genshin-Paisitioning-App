use std::sync::mpsc;
use tray_item::TrayItem;

enum Message {
    Quit,
}

pub fn add_tray_item() {
    let mut tray = TrayItem::new(env!("CARGO_PKG_DESCRIPTION"), "app-icon").unwrap();

    tray.add_label(env!("CARGO_PKG_DESCRIPTION")).unwrap();

    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("종료", move || {
        log::debug!("트레이로 부터 종료");
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => super::terminate_process(),
            _ => {}
        }
    }
}

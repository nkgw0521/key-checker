// src-tauri/src/lib.rs
//
// rdev を使ったキーボード入力チェッカー
// Linux / Windows 両対応
// rdev::listen() はブロッキングのため専用スレッドで起動する

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

// -----------------------------------------------------------------------
// 共通データ型
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub keys: Vec<String>,
    pub state: String,
    pub trigger_key: String,
}

type HeldKeys = Arc<Mutex<BTreeSet<String>>>;

// -----------------------------------------------------------------------
// Tauri コマンド
// -----------------------------------------------------------------------

#[tauri::command]
fn get_held_keys(state: tauri::State<HeldKeys>) -> Vec<String> {
    state.lock().unwrap().iter().cloned().collect()
}

// -----------------------------------------------------------------------
// Tauri アプリエントリ
// -----------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let held: HeldKeys = Arc::new(Mutex::new(BTreeSet::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(held.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let held2  = held.clone();

            // rdev::listen はブロッキングのため専用スレッドで起動
            std::thread::Builder::new()
                .name("rdev-listen".into())
                .spawn(move || {
                    key_listener::run(handle, held2);
                })
                .expect("Failed to spawn rdev-listen thread");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_held_keys])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// -----------------------------------------------------------------------
// ヘルパー：修飾キー順ソート
// -----------------------------------------------------------------------

pub fn sorted_keys(set: &BTreeSet<String>) -> Vec<String> {
    let modifier_order = ["Ctrl", "Alt", "Shift", "Meta", "CapsLock", "NumLock"];
    let mut modifiers: Vec<String> = Vec::new();
    let mut others: Vec<String> = Vec::new();

    for key in set.iter() {
        if modifier_order.contains(&key.as_str()) {
            modifiers.push(key.clone());
        } else {
            others.push(key.clone());
        }
    }

    modifiers.sort_by_key(|k| {
        modifier_order
            .iter()
            .position(|m| m == k)
            .unwrap_or(usize::MAX)
    });
    others.sort();
    modifiers.extend(others);
    modifiers
}

// -----------------------------------------------------------------------
// キーリスナー (rdev)
// -----------------------------------------------------------------------

mod key_listener {
    use super::*;
    use rdev::{listen, Event, EventType, Key};

    pub fn run(handle: AppHandle, held: HeldKeys) {
        if let Err(e) = listen(move |event: Event| {
            handle_event(&event, &handle, &held);
        }) {
            eprintln!("[key-checker] rdev listen error: {:?}", e);
        }
    }

    fn handle_event(event: &Event, handle: &AppHandle, held: &HeldKeys) {
        let (key_name, is_down) = match &event.event_type {
            EventType::KeyPress(k)   => (key_to_name(k), true),
            EventType::KeyRelease(k) => (key_to_name(k), false),
            _ => return,
        };

        let mut set = held.lock().unwrap();
        if is_down {
            set.insert(key_name.clone());
        } else {
            set.remove(&key_name);
        }

        let payload = KeyEvent {
            keys: super::sorted_keys(&set),
            state: if is_down { "down" } else { "up" }.to_string(),
            trigger_key: key_name,
        };
        let _ = handle.emit("key-event", payload);
    }

    fn key_to_name(key: &Key) -> String {
        match key {
            Key::KeyA => "A".into(), Key::KeyB => "B".into(),
            Key::KeyC => "C".into(), Key::KeyD => "D".into(),
            Key::KeyE => "E".into(), Key::KeyF => "F".into(),
            Key::KeyG => "G".into(), Key::KeyH => "H".into(),
            Key::KeyI => "I".into(), Key::KeyJ => "J".into(),
            Key::KeyK => "K".into(), Key::KeyL => "L".into(),
            Key::KeyM => "M".into(), Key::KeyN => "N".into(),
            Key::KeyO => "O".into(), Key::KeyP => "P".into(),
            Key::KeyQ => "Q".into(), Key::KeyR => "R".into(),
            Key::KeyS => "S".into(), Key::KeyT => "T".into(),
            Key::KeyU => "U".into(), Key::KeyV => "V".into(),
            Key::KeyW => "W".into(), Key::KeyX => "X".into(),
            Key::KeyY => "Y".into(), Key::KeyZ => "Z".into(),
            Key::Num0 => "0".into(), Key::Num1 => "1".into(),
            Key::Num2 => "2".into(), Key::Num3 => "3".into(),
            Key::Num4 => "4".into(), Key::Num5 => "5".into(),
            Key::Num6 => "6".into(), Key::Num7 => "7".into(),
            Key::Num8 => "8".into(), Key::Num9 => "9".into(),
            Key::F1  => "F1".into(),  Key::F2  => "F2".into(),
            Key::F3  => "F3".into(),  Key::F4  => "F4".into(),
            Key::F5  => "F5".into(),  Key::F6  => "F6".into(),
            Key::F7  => "F7".into(),  Key::F8  => "F8".into(),
            Key::F9  => "F9".into(),  Key::F10 => "F10".into(),
            Key::F11 => "F11".into(), Key::F12 => "F12".into(),
            Key::ControlLeft | Key::ControlRight => "Ctrl".into(),
            Key::ShiftLeft   | Key::ShiftRight   => "Shift".into(),
            Key::Alt                             => "Alt".into(),
            Key::AltGr                           => "AltGr".into(),
            Key::MetaLeft    | Key::MetaRight    => "Meta".into(),
            Key::Space     => "Space".into(),
            Key::Return    => "Enter".into(),
            Key::Backspace => "Backspace".into(),
            Key::Tab       => "Tab".into(),
            Key::Escape    => "Esc".into(),
            Key::Insert    => "Insert".into(),
            Key::Delete    => "Delete".into(),
            Key::Home      => "Home".into(),
            Key::End       => "End".into(),
            Key::PageUp    => "PageUp".into(),
            Key::PageDown  => "PageDown".into(),
            Key::UpArrow   => "Up".into(),
            Key::DownArrow => "Down".into(),
            Key::LeftArrow => "Left".into(),
            Key::RightArrow => "Right".into(),
            Key::CapsLock  => "CapsLock".into(),
            Key::NumLock   => "NumLock".into(),
            Key::ScrollLock => "ScrollLock".into(),
            Key::PrintScreen => "PrintScreen".into(),
            Key::Pause     => "Pause".into(),
            Key::Minus     => "-".into(),
            Key::Equal     => "=".into(),
            Key::LeftBracket  => "[".into(),
            Key::RightBracket => "]".into(),
            Key::BackSlash => "\\".into(),
            Key::SemiColon => ";".into(),
            Key::Quote     => "'".into(),
            Key::BackQuote => "`".into(),
            Key::Comma     => ",".into(),
            Key::Dot       => ".".into(),
            Key::Slash     => "/".into(),
            Key::Kp0 => "KP0".into(), Key::Kp1 => "KP1".into(),
            Key::Kp2 => "KP2".into(), Key::Kp3 => "KP3".into(),
            Key::Kp4 => "KP4".into(), Key::Kp5 => "KP5".into(),
            Key::Kp6 => "KP6".into(), Key::Kp7 => "KP7".into(),
            Key::Kp8 => "KP8".into(), Key::Kp9 => "KP9".into(),
            Key::KpReturn   => "KPEnter".into(),
            Key::KpPlus     => "KP+".into(),
            Key::KpMinus    => "KP-".into(),
            Key::KpMultiply => "KP*".into(),
            Key::KpDivide   => "KP/".into(),
            Key::KpDelete   => "KP.".into(),
            Key::Unknown(n) => format!("Unknown({n})"),
            _ => format!("{:?}", key),
        }
    }
}

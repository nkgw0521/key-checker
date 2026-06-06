// src-tauri/src/lib.rs
//
// キーボード入力チェッカー コアロジック
//
// Linux : evdev で /dev/input/event* を監視（要 input グループ or root）
// Windows: SetWindowsHookExW (WH_KEYBOARD_LL) で低レベルフック
//
// フロントエンドへは Tauri の emit で "key-event" イベントを送出する。
// ペイロード:
//   { keys: ["Ctrl", "Shift", "Insert"], state: "down" | "up" }

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

// -----------------------------------------------------------------------
// 共通データ型
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    /// 現在押されているキーのリスト（修飾キー先頭、アルファベット順）
    pub keys: Vec<String>,
    /// "down" | "up"
    pub state: String,
    /// 最後に操作されたキー名
    pub trigger_key: String,
}

/// アプリ全体で共有する「現在押下中キー集合」
type HeldKeys = Arc<Mutex<BTreeSet<String>>>;

// -----------------------------------------------------------------------
// Tauri コマンド（フロントから呼び出し可能）
// -----------------------------------------------------------------------

/// 現在押されているキー一覧を返す（ポーリング用）
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
            let held2 = held.clone();

            // OS ごとのフックをバックグラウンドスレッドで起動
            #[cfg(target_os = "linux")]
            {
                let handle2 = handle.clone();
                std::thread::spawn(move || {
                    linux_hook::run(handle2, held2);
                });
            }

            #[cfg(target_os = "windows")]
            {
                let handle2 = handle.clone();
                std::thread::spawn(move || {
                    windows_hook::run(handle2, held2);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_held_keys])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// -----------------------------------------------------------------------
// ヘルパー：キー名の正規化と修飾キー順ソート
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

    // 修飾キーを定義順に並べ替え
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
// Linux 実装 (evdev)
// -----------------------------------------------------------------------
#[cfg(target_os = "linux")]
pub mod linux_hook {
    use super::*;
    use evdev::{Device, EventType, InputEventKind, Key};
    use std::fs;

    /// /dev/input/event* からキーボードデバイスを列挙
    fn find_keyboard_devices() -> Vec<Device> {
        let mut devices = Vec::new();
        let Ok(entries) = fs::read_dir("/dev/input") else {
            return devices;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(dev) = Device::open(&path) {
                // EV_KEY をサポートするデバイスをキーボードとみなす
                if dev.supported_events().contains(EventType::KEY) {
                    devices.push(dev);
                }
            }
        }
        devices
    }

    pub fn run(handle: AppHandle, held: HeldKeys) {
        let mut devices = find_keyboard_devices();
        if devices.is_empty() {
            eprintln!("[key-checker] No keyboard devices found. Check /dev/input permissions.");
            return;
        }

        // 各デバイスを別スレッドで監視
        let mut handles = Vec::new();
        while let Some(mut dev) = devices.pop() {
            let handle2 = handle.clone();
            let held2 = held.clone();
            let h = std::thread::spawn(move || loop {
                let Ok(events) = dev.fetch_events() else {
                    break;
                };
                for ev in events {
                    if ev.event_type() != EventType::KEY {
                        continue;
                    }
                    let InputEventKind::Key(key) = ev.kind() else {
                        continue;
                    };
                    let value = ev.value(); // 0=up, 1=down, 2=repeat
                    if value == 2 {
                        continue; // リピートは無視
                    }
                    let key_name = key_to_name(key);
                    let state_str = if value == 1 { "down" } else { "up" };

                    {
                        let mut set = held2.lock().unwrap();
                        if value == 1 {
                            set.insert(key_name.clone());
                        } else {
                            set.remove(&key_name);
                        }
                        let payload = KeyEvent {
                            keys: super::sorted_keys(&set),
                            state: state_str.to_string(),
                            trigger_key: key_name,
                        };
                        let _ = handle2.emit("key-event", payload);
                    }
                }
            });
            handles.push(h);
        }
        for h in handles {
            let _ = h.join();
        }
    }

    /// evdev::Key → 表示用文字列
    fn key_to_name(key: Key) -> String {
        match key {
            Key::KEY_A => "A".into(),
            Key::KEY_B => "B".into(),
            Key::KEY_C => "C".into(),
            Key::KEY_D => "D".into(),
            Key::KEY_E => "E".into(),
            Key::KEY_F => "F".into(),
            Key::KEY_G => "G".into(),
            Key::KEY_H => "H".into(),
            Key::KEY_I => "I".into(),
            Key::KEY_J => "J".into(),
            Key::KEY_K => "K".into(),
            Key::KEY_L => "L".into(),
            Key::KEY_M => "M".into(),
            Key::KEY_N => "N".into(),
            Key::KEY_O => "O".into(),
            Key::KEY_P => "P".into(),
            Key::KEY_Q => "Q".into(),
            Key::KEY_R => "R".into(),
            Key::KEY_S => "S".into(),
            Key::KEY_T => "T".into(),
            Key::KEY_U => "U".into(),
            Key::KEY_V => "V".into(),
            Key::KEY_W => "W".into(),
            Key::KEY_X => "X".into(),
            Key::KEY_Y => "Y".into(),
            Key::KEY_Z => "Z".into(),
            Key::KEY_0 => "0".into(),
            Key::KEY_1 => "1".into(),
            Key::KEY_2 => "2".into(),
            Key::KEY_3 => "3".into(),
            Key::KEY_4 => "4".into(),
            Key::KEY_5 => "5".into(),
            Key::KEY_6 => "6".into(),
            Key::KEY_7 => "7".into(),
            Key::KEY_8 => "8".into(),
            Key::KEY_9 => "9".into(),
            Key::KEY_F1 => "F1".into(),
            Key::KEY_F2 => "F2".into(),
            Key::KEY_F3 => "F3".into(),
            Key::KEY_F4 => "F4".into(),
            Key::KEY_F5 => "F5".into(),
            Key::KEY_F6 => "F6".into(),
            Key::KEY_F7 => "F7".into(),
            Key::KEY_F8 => "F8".into(),
            Key::KEY_F9 => "F9".into(),
            Key::KEY_F10 => "F10".into(),
            Key::KEY_F11 => "F11".into(),
            Key::KEY_F12 => "F12".into(),
            Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => "Ctrl".into(),
            Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => "Shift".into(),
            Key::KEY_LEFTALT | Key::KEY_RIGHTALT => "Alt".into(),
            Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => "Meta".into(),
            Key::KEY_SPACE => "Space".into(),
            Key::KEY_ENTER => "Enter".into(),
            Key::KEY_BACKSPACE => "Backspace".into(),
            Key::KEY_TAB => "Tab".into(),
            Key::KEY_ESC => "Esc".into(),
            Key::KEY_INSERT => "Insert".into(),
            Key::KEY_DELETE => "Delete".into(),
            Key::KEY_HOME => "Home".into(),
            Key::KEY_END => "End".into(),
            Key::KEY_PAGEUP => "PageUp".into(),
            Key::KEY_PAGEDOWN => "PageDown".into(),
            Key::KEY_UP => "Up".into(),
            Key::KEY_DOWN => "Down".into(),
            Key::KEY_LEFT => "Left".into(),
            Key::KEY_RIGHT => "Right".into(),
            Key::KEY_CAPSLOCK => "CapsLock".into(),
            Key::KEY_NUMLOCK => "NumLock".into(),
            Key::KEY_SCROLLLOCK => "ScrollLock".into(),
            Key::KEY_PRINT => "PrintScreen".into(),
            Key::KEY_PAUSE => "Pause".into(),
            Key::KEY_MINUS => "-".into(),
            Key::KEY_EQUAL => "=".into(),
            Key::KEY_LEFTBRACE => "[".into(),
            Key::KEY_RIGHTBRACE => "]".into(),
            Key::KEY_BACKSLASH => "\\".into(),
            Key::KEY_SEMICOLON => ";".into(),
            Key::KEY_APOSTROPHE => "'".into(),
            Key::KEY_GRAVE => "`".into(),
            Key::KEY_COMMA => ",".into(),
            Key::KEY_DOT => ".".into(),
            Key::KEY_SLASH => "/".into(),
            Key::KEY_KP0 => "KP0".into(),
            Key::KEY_KP1 => "KP1".into(),
            Key::KEY_KP2 => "KP2".into(),
            Key::KEY_KP3 => "KP3".into(),
            Key::KEY_KP4 => "KP4".into(),
            Key::KEY_KP5 => "KP5".into(),
            Key::KEY_KP6 => "KP6".into(),
            Key::KEY_KP7 => "KP7".into(),
            Key::KEY_KP8 => "KP8".into(),
            Key::KEY_KP9 => "KP9".into(),
            Key::KEY_KPENTER => "KPEnter".into(),
            Key::KEY_KPPLUS => "KP+".into(),
            Key::KEY_KPMINUS => "KP-".into(),
            Key::KEY_KPASTERISK => "KP*".into(),
            Key::KEY_KPSLASH => "KP/".into(),
            Key::KEY_KPDOT => "KP.".into(),
            _ => format!("{:?}", key),
        }
    }
}

// -----------------------------------------------------------------------
// Windows 実装 (WH_KEYBOARD_LL)
// -----------------------------------------------------------------------
#[cfg(target_os = "windows")]
pub mod windows_hook {
    use super::*;
    use std::sync::OnceLock;
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    // グローバルコールバック用の静的ストレージ
    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static HELD_KEYS: OnceLock<HeldKeys> = OnceLock::new();

    pub fn run(handle: AppHandle, held: HeldKeys) {
        APP_HANDLE.get_or_init(|| handle);
        HELD_KEYS.get_or_init(|| held);

        unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0)
                .expect("Failed to set keyboard hook");

            // メッセージループ（フックに必要）
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        }
    }

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk = VIRTUAL_KEY(info.vkCode as u16);
            let key_name = vk_to_name(vk);

            let is_down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
            let is_up = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;

            if is_down || is_up {
                if let (Some(handle), Some(held)) = (APP_HANDLE.get(), HELD_KEYS.get()) {
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
            }
        }

        CallNextHookEx(None, code, wparam, lparam)
    }

    fn vk_to_name(vk: VIRTUAL_KEY) -> String {
        match vk {
            VK_LCONTROL | VK_RCONTROL | VK_CONTROL => "Ctrl".into(),
            VK_LSHIFT | VK_RSHIFT | VK_SHIFT => "Shift".into(),
            VK_LMENU | VK_RMENU | VK_MENU => "Alt".into(),
            VK_LWIN | VK_RWIN => "Meta".into(),
            VK_RETURN => "Enter".into(),
            VK_ESCAPE => "Esc".into(),
            VK_SPACE => "Space".into(),
            VK_BACK => "Backspace".into(),
            VK_TAB => "Tab".into(),
            VK_INSERT => "Insert".into(),
            VK_DELETE => "Delete".into(),
            VK_HOME => "Home".into(),
            VK_END => "End".into(),
            VK_PRIOR => "PageUp".into(),
            VK_NEXT => "PageDown".into(),
            VK_UP => "Up".into(),
            VK_DOWN => "Down".into(),
            VK_LEFT => "Left".into(),
            VK_RIGHT => "Right".into(),
            VK_CAPITAL => "CapsLock".into(),
            VK_NUMLOCK => "NumLock".into(),
            VK_SCROLL => "ScrollLock".into(),
            VK_SNAPSHOT => "PrintScreen".into(),
            VK_PAUSE => "Pause".into(),
            VK_F1 => "F1".into(),
            VK_F2 => "F2".into(),
            VK_F3 => "F3".into(),
            VK_F4 => "F4".into(),
            VK_F5 => "F5".into(),
            VK_F6 => "F6".into(),
            VK_F7 => "F7".into(),
            VK_F8 => "F8".into(),
            VK_F9 => "F9".into(),
            VK_F10 => "F10".into(),
            VK_F11 => "F11".into(),
            VK_F12 => "F12".into(),
            VK_NUMPAD0 => "KP0".into(),
            VK_NUMPAD1 => "KP1".into(),
            VK_NUMPAD2 => "KP2".into(),
            VK_NUMPAD3 => "KP3".into(),
            VK_NUMPAD4 => "KP4".into(),
            VK_NUMPAD5 => "KP5".into(),
            VK_NUMPAD6 => "KP6".into(),
            VK_NUMPAD7 => "KP7".into(),
            VK_NUMPAD8 => "KP8".into(),
            VK_NUMPAD9 => "KP9".into(),
            VK_MULTIPLY => "KP*".into(),
            VK_ADD => "KP+".into(),
            VK_SUBTRACT => "KP-".into(),
            VK_DECIMAL => "KP.".into(),
            VK_DIVIDE => "KP/".into(),
            _ => {
                // A-Z
                let c = vk.0 as u8;
                if (b'A'..=b'Z').contains(&c) {
                    return (c as char).to_string();
                }
                // 0-9
                if (b'0'..=b'9').contains(&c) {
                    return (c as char).to_string();
                }
                format!("VK_{:04X}", vk.0)
            }
        }
    }
}

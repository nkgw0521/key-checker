// src/main.js
// キー検出をWebView内のKeyboardEventで行う
// → 自アプリフォーカス時も確実に動作する
// → 他アプリフォーカス時はRust側(rdev)で補完

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// -----------------------------------------------------------------------
// DOM refs
// -----------------------------------------------------------------------
const statusDot   = document.getElementById("status-dot");
const hint        = document.getElementById("hint");
const keyBadges   = document.getElementById("key-badges");
const lastCombo   = document.getElementById("last-combo");
const logArea     = document.getElementById("log-area");
const logList     = document.getElementById("log-list");
const btnLog      = document.getElementById("btn-log");
const btnClear    = document.getElementById("btn-clear");
const displayArea = document.getElementById("key-display-area");

// -----------------------------------------------------------------------
// 状態
// -----------------------------------------------------------------------
const MODIFIERS = new Set(["Ctrl","Alt","Shift","Meta","CapsLock","NumLock","ScrollLock"]);
let logVisible  = true;
let fadeTimer   = null;
let maxLogLines = 200;

// 現在押されているキーのセット（JS側で管理）
const heldKeys = new Set();

// -----------------------------------------------------------------------
// ログ表示切替
// -----------------------------------------------------------------------
btnLog.addEventListener("click", () => {
  logVisible = !logVisible;
  logArea.classList.toggle("collapsed", !logVisible);
  btnLog.classList.toggle("active-btn", logVisible);
});

btnClear.addEventListener("click", () => {
  logList.innerHTML = "";
});

// -----------------------------------------------------------------------
// キーバッジ描画
// -----------------------------------------------------------------------
function sortedKeys(keys) {
  const modOrder = ["Ctrl","Alt","Shift","Meta","CapsLock","NumLock"];
  const mods = [];
  const others = [];
  for (const k of keys) {
    if (modOrder.includes(k)) mods.push(k);
    else others.push(k);
  }
  mods.sort((a,b) => modOrder.indexOf(a) - modOrder.indexOf(b));
  others.sort();
  return [...mods, ...others];
}

function renderBadges(keys) {
  keyBadges.innerHTML = "";
  if (keys.length === 0) {
    hint.style.display = "";
    return;
  }
  hint.style.display = "none";

  keys.forEach((k, i) => {
    if (i > 0) {
      const sep = document.createElement("span");
      sep.className = "key-sep";
      sep.textContent = "+";
      keyBadges.appendChild(sep);
    }
    const badge = document.createElement("span");
    badge.className = "key-badge" + (MODIFIERS.has(k) ? " modifier" : "");
    badge.textContent = k;
    keyBadges.appendChild(badge);
  });
}

// -----------------------------------------------------------------------
// フェードアウトタイマー
// -----------------------------------------------------------------------
function scheduleHide() {
  if (fadeTimer) clearTimeout(fadeTimer);
  displayArea.classList.add("fading");
  fadeTimer = setTimeout(() => {
    renderBadges([]);
    displayArea.classList.remove("fading");
    fadeTimer = null;
  }, 800);
}

function cancelHide() {
  if (fadeTimer) { clearTimeout(fadeTimer); fadeTimer = null; }
  displayArea.classList.remove("fading");
}

// -----------------------------------------------------------------------
// ログ追記
// -----------------------------------------------------------------------
function addLog(keys, triggerKey) {
  const now = new Date();
  const ts  = now.toTimeString().slice(0,8) + "." +
              String(now.getMilliseconds()).padStart(3,"0");

  const li = document.createElement("li");
  const tsSpan    = document.createElement("span");
  tsSpan.className = "ts";
  tsSpan.textContent = ts;

  const dirSpan   = document.createElement("span");
  dirSpan.className = "dir-down";
  dirSpan.textContent = "▼";

  const comboSpan = document.createElement("span");
  comboSpan.className = "combo-text";
  comboSpan.textContent = keys.length > 0 ? keys.join(" + ") : triggerKey;

  li.appendChild(tsSpan);
  li.appendChild(dirSpan);
  li.appendChild(comboSpan);
  logList.prepend(li);

  while (logList.children.length > maxLogLines) {
    logList.removeChild(logList.lastChild);
  }
}

// -----------------------------------------------------------------------
// キー名の正規化（KeyboardEvent.code → 表示名）
// -----------------------------------------------------------------------
function normalizeKey(e) {
  // 修飾キー
  if (e.code === "ControlLeft"  || e.code === "ControlRight") return "Ctrl";
  if (e.code === "ShiftLeft"    || e.code === "ShiftRight")   return "Shift";
  if (e.code === "AltLeft")     return "Alt";
  if (e.code === "AltRight")    return "AltGr";
  if (e.code === "MetaLeft"     || e.code === "MetaRight")    return "Meta";
  if (e.code === "CapsLock")    return "CapsLock";
  if (e.code === "NumLock")     return "NumLock";
  if (e.code === "ScrollLock")  return "ScrollLock";

  // ファンクションキー
  if (/^F(\d+)$/.test(e.key))  return e.key;

  // 特殊キー
  const specials = {
    " ": "Space", "Enter": "Enter", "Backspace": "Backspace",
    "Tab": "Tab", "Escape": "Esc", "Insert": "Insert",
    "Delete": "Delete", "Home": "Home", "End": "End",
    "PageUp": "PageUp", "PageDown": "PageDown",
    "ArrowUp": "Up", "ArrowDown": "Down",
    "ArrowLeft": "Left", "ArrowRight": "Right",
    "PrintScreen": "PrintScreen", "Pause": "Pause",
  };
  if (specials[e.key]) return specials[e.key];

  // テンキー
  if (e.code.startsWith("Numpad")) {
    const numpadMap = {
      "Numpad0":"KP0","Numpad1":"KP1","Numpad2":"KP2","Numpad3":"KP3",
      "Numpad4":"KP4","Numpad5":"KP5","Numpad6":"KP6","Numpad7":"KP7",
      "Numpad8":"KP8","Numpad9":"KP9","NumpadEnter":"KPEnter",
      "NumpadAdd":"KP+","NumpadSubtract":"KP-",
      "NumpadMultiply":"KP*","NumpadDivide":"KP/","NumpadDecimal":"KP.",
    };
    if (numpadMap[e.code]) return numpadMap[e.code];
  }

  // 通常キー：1文字なら大文字で返す
  if (e.key.length === 1) return e.key.toUpperCase();

  return e.key;
}

// -----------------------------------------------------------------------
// WebView KeyboardEvent（自アプリフォーカス時）
// -----------------------------------------------------------------------
window.addEventListener("keydown", (e) => {
  e.preventDefault(); // WebViewへのキー入力を防ぐ
  const keyName = normalizeKey(e);
  if (heldKeys.has(keyName)) return; // リピート無視

  heldKeys.add(keyName);
  const keys = sortedKeys([...heldKeys]);

  cancelHide();
  renderBadges(keys);
  lastCombo.textContent = keys.join(" + ");
  addLog(keys, keyName);
  statusDot.classList.add("active");
}, true);

window.addEventListener("keyup", (e) => {
  e.preventDefault();
  const keyName = normalizeKey(e);
  heldKeys.delete(keyName);
  const keys = sortedKeys([...heldKeys]);

  if (keys.length > 0) {
    renderBadges(keys);
    lastCombo.textContent = keys.join(" + ");
  } else {
    scheduleHide();
  }
}, true);

// フォーカスを失ったときに heldKeys をクリア（他アプリに切り替え時）
window.addEventListener("blur", () => {
  heldKeys.clear();
});

// -----------------------------------------------------------------------
// Tauri イベント受信（他アプリフォーカス時 = Rust側rdevから）
// -----------------------------------------------------------------------
async function startListening() {
  try {
    await listen("key-event", (event) => {
      const { keys, state, trigger_key } = event.payload;
      statusDot.classList.add("active");

      if (state === "down") {
        cancelHide();
        renderBadges(keys);
        if (keys.length > 0) {
          lastCombo.textContent = keys.join(" + ");
          addLog(keys, trigger_key);
        }
      } else {
        if (keys.length > 0) {
          renderBadges(keys);
          lastCombo.textContent = keys.join(" + ");
        } else {
          scheduleHide();
        }
      }
    });
    statusDot.classList.add("active");
  } catch (e) {
    console.error("[key-checker] Failed to listen:", e);
  }
}

startListening();

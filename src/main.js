// src/main.js
// Tauri v2 の @tauri-apps/api/event を使用

import { listen } from "@tauri-apps/api/event";

// -----------------------------------------------------------------------
// DOM refs
// -----------------------------------------------------------------------
const statusDot    = document.getElementById("status-dot");
const hint         = document.getElementById("hint");
const keyBadges    = document.getElementById("key-badges");
const lastCombo    = document.getElementById("last-combo");
const logArea      = document.getElementById("log-area");
const logList      = document.getElementById("log-list");
const btnLog       = document.getElementById("btn-log");
const btnClear     = document.getElementById("btn-clear");
const displayArea  = document.getElementById("key-display-area");

// -----------------------------------------------------------------------
// 状態
// -----------------------------------------------------------------------
const MODIFIERS = new Set(["Ctrl", "Alt", "Shift", "Meta", "CapsLock", "NumLock", "ScrollLock"]);
let logVisible  = true;
let fadeTimer   = null;
let maxLogLines = 200;

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
// フェードアウトタイマー（全キーが離されたら 0.8s 後にクリア）
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
  if (fadeTimer) {
    clearTimeout(fadeTimer);
    fadeTimer = null;
  }
  displayArea.classList.remove("fading");
}

// -----------------------------------------------------------------------
// ログ追記
// -----------------------------------------------------------------------
function addLog(keys, state, triggerKey) {
  const now = new Date();
  const ts  = now.toTimeString().slice(0, 8) + "." +
              String(now.getMilliseconds()).padStart(3, "0");

  const li = document.createElement("li");

  const tsSpan = document.createElement("span");
  tsSpan.className = "ts";
  tsSpan.textContent = ts;

  const dirSpan = document.createElement("span");
  dirSpan.className = state === "down" ? "dir-down" : "dir-up";
  dirSpan.textContent = state === "down" ? "▼" : "▲";

  const comboSpan = document.createElement("span");
  comboSpan.className = "combo-text";
  comboSpan.textContent = keys.length > 0 ? keys.join(" + ") : triggerKey;

  li.appendChild(tsSpan);
  li.appendChild(dirSpan);
  li.appendChild(comboSpan);
  logList.prepend(li); // 最新を先頭に

  // 上限を超えたら古いエントリを削除
  while (logList.children.length > maxLogLines) {
    logList.removeChild(logList.lastChild);
  }
}

// -----------------------------------------------------------------------
// Tauri イベント受信
// -----------------------------------------------------------------------
async function startListening() {
  try {
    await listen("key-event", (event) => {
      const { keys, state, trigger_key } = event.payload;

      statusDot.classList.add("active");

      if (state === "down") {
        cancelHide();
        renderBadges(keys);

        // 最後のコンボをテキストで小さく表示
        if (keys.length > 0) {
          lastCombo.textContent = keys.join(" + ");
        }
      } else {
        // up: まだ押されているキーがあれば更新、なければフェード
        if (keys.length > 0) {
          renderBadges(keys);
          lastCombo.textContent = keys.join(" + ");
        } else {
          scheduleHide();
        }
      }

      // ログは down のみ記録（ノイズを減らす）
      if (state === "down") {
        addLog(keys, state, trigger_key);
      }
    });

    statusDot.classList.add("active");
    console.log("[key-checker] Listening for key events.");
  } catch (e) {
    console.error("[key-checker] Failed to listen:", e);
    statusDot.classList.remove("active");
  }
}

startListening();

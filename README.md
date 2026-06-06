# key-checker

自作キーボードなどの入力確認ツール。  
押したキーをリアルタイムで大きく表示し、同時押し（Ctrl+Shift+Ins など）にも対応。

## 構成

```
key-checker/
├── index.html              # フロントエンド HTML/CSS
├── src/
│   └── main.js             # Tauri イベント受信 + UI ロジック
├── vite.config.js
├── package.json
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    └── src/
        ├── main.rs         # エントリポイント
        └── lib.rs          # キーフック実装 (Linux/Windows)
```

## 動作原理

| OS      | 方式                                | 備考                                      |
|---------|-------------------------------------|-------------------------------------------|
| Linux   | `evdev` で `/dev/input/event*` を読む | `input` グループへの追加 or sudo が必要    |
| Windows | `SetWindowsHookExW(WH_KEYBOARD_LL)` | 管理者権限不要（通常ユーザーで動作）        |

## ビルド手順

### 必要ツール

- [Rust (stable)](https://rustup.rs/)
- Node.js >= 18
- Tauri v2 の依存ライブラリ（下記参照）

### Linux 追加依存

```bash
# Debian/Ubuntu 系
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Fedora 系
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
```

### /dev/input 権限設定（Linux）

```bash
# 現在のユーザーを input グループに追加（再ログインが必要）
sudo usermod -aG input $USER

# 確認
groups
```

### ビルド & 起動

```bash
# 依存インストール
npm install

# 開発モード（ホットリロード）
npm run tauri dev

# リリースビルド
npm run tauri build
# → src-tauri/target/release/bundle/ 以下に成果物が生成される
```

WindowsへのクロスコンパイルはLinuxからだと現実的に難しいです。**Windows環境でビルドするのが確実**です。

## Windows環境でのビルド手順

### 1. 必要ツールのインストール

**Rust**
```powershell
# https://rustup.rs/ からインストーラをダウンロードして実行
# インストール時のオプションはデフォルトでOK
rustup --version  # 確認
```

**Node.js**
```powershell
# https://nodejs.org/ からLTS版をダウンロードして実行
node --version  # 確認
```

**WebView2ランタイム**（Windows 11は標準搭載、Windows 10は要インストール）
```
https://developer.microsoft.com/ja-jp/microsoft-edge/webview2/
```

**Visual Studio Build Tools**（Rustのコンパイルに必要）
```
https://visualstudio.microsoft.com/ja/visual-cpp-build-tools/
インストール時に「C++によるデスクトップ開発」にチェック
```

---

### 2. リポジトリをクローン

```powershell
git clone git@github.com:nkgw0521/key-checker.git
cd key-checker
```

---

### 3. ビルド

```powershell
npm install
npm run tauri build
```

成果物は以下に生成されます：
```
src-tauri/target/release/bundle/
├── msi/
│   └── key-checker_0.1.0_x64_en-US.msi   # インストーラ
└── nsis/
    └── key-checker_0.1.0_x64-setup.exe    # インストーラ
```

## UI 説明

```
┌─────────────────────────────────────┐
│ ● Key Checker          [LOG] [CLEAR] │  ← ヘッダー（● = 接続状態）
├─────────────────────────────────────┤
│                                     │
│   [Ctrl]  +  [Shift]  +  [Insert]   │  ← リアルタイム大表示
│                                     │
│      Ctrl + Shift + Insert          │  ← 最後のコンボ（小）
├─────────────────────────────────────┤
│ 12:34:56.789  ▼  Ctrl + Shift + Insert │  ← ログ（最新順）
│ 12:34:55.100  ▼  A                  │
└─────────────────────────────────────┘
```

- 修飾キー（Ctrl/Shift/Alt/Meta）は紫、通常キーは水色で表示
- 全キーを離してから約 0.8 秒後にフェードアウト
- ログは最大 200 行保持（[CLEAR] で消去）

## カスタマイズ

### フェードアウト時間を変える

`src/main.js` の `scheduleHide()` 内の `800` (ms) を変更。

### ログ上限を変える

`src/main.js` の `maxLogLines = 200` を変更。

### ウィンドウサイズ・常前面を変える

`src-tauri/tauri.conf.json` の `windows` セクションを編集：

```json
"alwaysOnTop": true,   // 常前面
"width": 600,
"height": 400
```

## トラブルシューティング

### Linux: キーが検出されない

```bash
# /dev/input の権限確認
ls -la /dev/input/event*

# input グループ確認
groups

# デバイス確認（evtest が使える場合）
evtest
```

### Windows: フックが取れない

- セキュリティソフトがフックをブロックしている場合があります
- UAC を求められる場合は「はい」で続行してください

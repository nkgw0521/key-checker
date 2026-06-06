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

| OS      | 方式                                | 備考                                          |
|---------|-------------------------------------|-----------------------------------------------|
| Linux   | `evdev` で `/dev/input/event*` を読む | udevルール設定 or input グループへの追加が必要 |
| Windows | `SetWindowsHookExW(WH_KEYBOARD_LL)` | 管理者権限不要（通常ユーザーで動作）            |

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

**udevルールで設定する（推奨）**

```bash
sudo tee /etc/udev/rules.d/99-key-checker.rules << 'EOF'
KERNEL=="event*", SUBSYSTEM=="input", GROUP="input", MODE="0664"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger

# 確認（crw-rw-r-- となっていればOK）
ls -la /dev/input/event0
```

**inputグループに追加する場合**

```bash
sudo usermod -aG input $USER

# グループへの追加を確認
grep input /etc/group
# → input:x:996:ユーザー名 と出ればOK

# ディスプレイマネージャを再起動（セッションが終了するので作業を保存してから）
sudo systemctl restart gdm3    # GNOMEの場合
sudo systemctl restart lightdm # LXDEなどの場合

# 再ログイン後に確認
groups  # input が含まれていればOK
```

> **注意**: `usermod` 後に `groups` へ反映されない場合はudevルール方式を使ってください。
> PAMの設定によってはディスプレイマネージャ再起動後も反映されないことがあります。

### アイコンファイルの生成

`tauri::generate_context!()` はコンパイル時にアイコンファイルを要求します。
`npm run tauri icon` コマンドで生成するのが正規の方法ですが、元画像がない場合は以下で最小限のアイコンを生成できます。

```bash
# npm install 後に実行
npm install

# 元画像（1024x1024推奨）から全サイズ自動生成
npm run tauri icon path/to/your-icon.png

# 元画像がない場合はPythonで生成
python3 - <<'EOF'
import struct, zlib, os, shutil

os.makedirs("src-tauri/icons", exist_ok=True)

def make_rgba_png(size):
    def chunk(name, data):
        c = zlib.crc32(name + data) & 0xFFFFFFFF
        return struct.pack('>I', len(data)) + name + data + struct.pack('>I', c)
    sig = b'\x89PNG\r\n\x1a\n'
    ihdr = chunk(b'IHDR', struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0))
    raw = b''.join(b'\x00' + bytes([110, 231, 247, 255] * size) for _ in range(size))
    idat = chunk(b'IDAT', zlib.compress(raw))
    iend = chunk(b'IEND', b'')
    return sig + ihdr + idat + iend

for name, sz in [("32x32", 32), ("128x128", 128), ("128x128@2x", 256)]:
    with open(f"src-tauri/icons/{name}.png", "wb") as f:
        f.write(make_rgba_png(sz))

shutil.copy("src-tauri/icons/128x128.png", "src-tauri/icons/icon.icns")
shutil.copy("src-tauri/icons/32x32.png",   "src-tauri/icons/icon.ico")
print("Icons generated")
EOF
```

> **注意**: PNGはRGBA形式である必要があります。RGB形式だとコンパイルエラーになります。

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

### 2. リポジトリをクローン

```powershell
git clone git@github.com:nkgw0521/key-checker.git
cd key-checker
```

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

# inputグループへの追加確認（groupsコマンドではなくこちらで確認）
grep input /etc/group

# udevルールが適用されているか確認
cat /etc/udev/rules.d/99-key-checker.rules

# デバイス確認（evtest が使える場合）
evtest
```

### Linux: groupsにinputが反映されない

`usermod` でグループ追加済みでも `groups` に反映されない場合があります（PAMの設定による）。
その場合はudevルール方式で対処してください（「/dev/input 権限設定」の推奨手順を参照）。

### Linux: debパッケージでインストールすると動かない

debインストール後にランチャーから起動した場合も同様の権限問題が起きます。
udevルールを適用することで解決します。

### Windows: フックが取れない

- セキュリティソフトがフックをブロックしている場合があります
- UAC を求められる場合は「はい」で続行してください

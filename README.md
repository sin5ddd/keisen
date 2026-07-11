# keisen

> **Japanese version is below.** / **日本語は下へ**

Windows floating palette for box-drawing characters (keisen).

Click a character like `├` `│` `└` and it is typed into the currently focused editor.

```
└─┬─ tree-style notes
  └─ type them quickly
```

## Features

- Always-on-top borderless float window (no taskbar button)
- Single vertical scroll (no tabs)
- Order: **Frequent → Light → Heavy → Double → Mixed**
- Frequent chars first: `├ │ └ ─ ┬`
- Covers light/heavy mixes (e.g. `┨`) and double lines
- Direct input via **Win32 SendInput** (does not use the clipboard)
- Drag the title area to move; resize from the bottom-right

## Requirements

| Item | Detail |
|------|--------|
| OS | Windows |
| Build | [Rust](https://www.rust-lang.org/) (edition 2024) |
| Run | Prebuilt `keisen.exe` is enough (no extra runtime) |

## Run

```bash
cargo run --release
```

Build only:

```bash
cargo build --release
```

Output: `target/release/keisen.exe`

### Prebuilt releases

GitHub Actions builds a Windows binary when you push a version tag:

```bash
git tag v0.1.1
git push origin v0.1.1
```

The workflow creates a [Release](https://github.com/sin5ddd/keisen/releases) with:

- `keisen.exe`
- `keisen-<version>-windows-x64.zip` (exe + LICENSE + README)

You can also run **Release** from the Actions tab (`workflow_dispatch`) and pass an existing tag (e.g. `v0.1.0`).

## Usage

1. Put the caret in Notepad, VS Code, or any editor
2. Click a character button in keisen
3. The character is inserted into the previously active editor

Drag the empty area of the title bar to move. Press `×` to quit.

> **Note:** With Japanese IME on, some apps may still open conversion candidates. Switch to alphanumeric mode temporarily if needed.

## Customize characters

All glyphs are defined in `src/chars.rs` as `SECTIONS`.

Edit that array to add sections, reorder them, or change characters — the UI updates automatically.

```rust
Section {
    title: "よく使う",
    chars: &['├', '│', '└', '─', '┬'],
},
```

## Project layout

```
keisen/
├── Cargo.toml
├── README.md
├── assets/
│   ├── icon.png / icon.ico   # app icon (dark keycap + 田)
│   └── generate_icon.py      # regenerate icons
└── src/
    ├── main.rs    # window & UI
    ├── chars.rs   # character data
    └── input.rs   # send keys to foreground window
```

## Stack

- [eframe / egui](https://github.com/emilk/egui) — UI
- [windows](https://github.com/microsoft/windows-rs) crate — Win32 `SendInput`, etc.

## License

[MIT](LICENSE)

---

# keisen（日本語）

Windows 向けの **罫線入力フローティングアプリ**。

常時最前面の小さなパレットから、ボックス描画文字（`├` `│` `└` など）をワンクリックで、いまフォーカスしているエディタへ直接入力します。

```
└─┬─ こういうツリー記法
  └─ サッと打てます
```

## 機能

- **常時最前面**の枠なしフロートウィンドウ（タスクバー非表示）
- **縦スクロール**で全文字を一覧（タブなし）
- 並び順: **よく使う → 細 → 太 → 二重 → 混**
- 先頭に使用頻度の高い `├ │ └ ─ ┬`
- 細・太の組み合わせ（例: `┨`）や二重線も収録
- **Win32 SendInput** でカーソル位置へ直接入力（クリップボードは使わない）
- タイトルバーをドラッグで移動、右下でリサイズ

## 必要環境

| 項目 | 内容 |
|------|------|
| OS | Windows |
| ビルド | [Rust](https://www.rust-lang.org/)（edition 2024） |
| 実行 | ビルド済み `keisen.exe` のみでも可（ランタイム不要） |

## 起動

```bash
cargo run --release
```

ビルドだけする場合:

```bash
cargo build --release
```

成果物: `target/release/keisen.exe`

### ビルド済みリリース

バージョンタグを push すると GitHub Actions が Windows 用バイナリをビルドします。

```bash
git tag v0.1.1
git push origin v0.1.1
```

[Releases](https://github.com/sin5ddd/keisen/releases) に次が添付されます。

- `keisen.exe`
- `keisen-<version>-windows-x64.zip`（exe + LICENSE + README）

Actions タブから **Release** を手動実行（`workflow_dispatch`）し、既存タグ（例: `v0.1.0`）を指定することもできます。

## 使い方

1. メモ帳・VS Code など、入力したいアプリにカーソルを置く
2. keisen の文字ボタンをクリック
3. アクティブだったエディタに、その文字が入力される

ウィンドウ上部の余白をドラッグすると移動できます。`×` で終了です。

> **メモ:** 日本語 IME が ON のとき、アプリによっては変換候補に載ることがあります。必要なら一時的に英数モードに切り替えてください。

## 文字セットのカスタム

表示する文字は `src/chars.rs` の `SECTIONS` で定義しています。

- セクションの追加・並べ替え
- 各セクション内の文字の追加・順序変更

は、この配列を編集するだけで UI に反映されます。

```rust
Section {
    title: "よく使う",
    chars: &['├', '│', '└', '─', '┬'],
},
```

## プロジェクト構成

```
keisen/
├── Cargo.toml
├── README.md
├── assets/
│   ├── icon.png / icon.ico   # アプリアイコン（キートップ + 田）
│   └── generate_icon.py      # アイコン再生成
└── src/
    ├── main.rs    # ウィンドウ・UI
    ├── chars.rs   # 罫線文字データ
    └── input.rs   # 前面ウィンドウへの文字送信
```

## 技術スタック

- [eframe / egui](https://github.com/emilk/egui) — UI
- [windows](https://github.com/microsoft/windows-rs) crate — `SendInput` など Win32 API

## ライセンス

[MIT](LICENSE)

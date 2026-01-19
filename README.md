# BDD+

Rust 製のオーバードライブ系オーディオプラグインです。  
VST3/CLAP に対応し、Cubase 13（Windows）で動作確認しています。

## ビルド

Rust をインストールした上で、以下を実行してください。

```shell
cargo xtask bundle bdd_plus --release
```

## 仕様

- パラメータ: Drive / Tone / Level / Bypass（ホストバイパス連携）
- フォーマット: VST3 / CLAP
- GUI: egui ベースのカスタム GUI（背景テクスチャ + 3 縦スライダー）

## 構成

- `src/lib.rs`: プラグイン本体
- `src/params.rs`: パラメータ定義
- `src/dsp/`: DSP 実装
- `src/gui/`: GUI 実装
- `assets/bg.png`: 背景テクスチャ

## 開発メモ

- nih-plug を使用
- 2x オーバーサンプリングを前提とした歪み構成

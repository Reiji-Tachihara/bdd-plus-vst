# bdd_plus

## Building

After installing [Rust](https://rustup.rs/), you can compile bdd_plus as follows:

```shell
cargo xtask bundle bdd_plus --release
```

# BDD+

Rust で実装した VST3 オーディオプラグイン（歪みエフェクター）。
Cubase 13 で動作確認済み。

## Features

- Drive / Tone / Level
- VST3 / CLAP 対応
- Rust + nih-plug + egui

## Why Rust?

- メモリ安全性
- リアルタイム DSP での安心感
- C++依存を最小化

## What I learned

- Rust によるオーディオ DSP 設計
- xtask によるビルド自動化
- Windows / MSVC 環境での Rust 開発

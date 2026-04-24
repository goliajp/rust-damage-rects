# damage-rects

[![Crates.io](https://img.shields.io/crates/v/damage-rects?style=flat-square&logo=rust)](https://crates.io/crates/damage-rects)
[![docs.rs](https://img.shields.io/docsrs/damage-rects?style=flat-square&logo=docs.rs)](https://docs.rs/damage-rects)
[![License](https://img.shields.io/crates/l/damage-rects?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.94-blue?style=flat-square&logo=rust)](Cargo.toml)
[![Downloads](https://img.shields.io/crates/d/damage-rects?style=flat-square)](https://crates.io/crates/damage-rects)

[English](README.md) | [简体中文](README.zh-CN.md) | **日本語**

ダーティ矩形を集めて、GPU キャンバスの部分更新のためのマージ済み再描画領域を生成します。依存ゼロ、GPU との結合なし、ビューポートの仮定もなし——任意の 2D レンダラ（Skia、wgpu、softbuffer、生 Metal）と組み合わせ可能。

> **同義語 / 検索ワード**: damage tracking、ダーティ矩形トラッキング、部分再描画（partial redraw）、無効化領域（invalidation region）、再描画領域、領域合流、エディタ/ターミナルのインクリメンタル再描画。

## なぜ

インタラクティブな GPU レンダリングアプリはどれも選択を迫られます: 毎ティック全体を再描画するか、どの領域が変わったかを追跡してそこだけ再描画するか。前者は単純だが電力を浪費し、後者はエディタ、デザインツール、ターミナル、ダッシュボードが実際に行っていることです。

GUI フレームワーク（`gpui`、`iced`、`slint`）は damage トラッキングを内部に抱え込み、再利用可能な crate として公開しません。この crate がその再利用可能な実装です。

## API

```rust
use damage_rects::{DamageRect, DamageTracker};

let mut tracker = DamageTracker::new();

// 状態更新時
tracker.add(DamageRect::new(10.0, 20.0, 100.0, 30.0)); // ある行が変化
tracker.add(DamageRect::new(50.0, 30.0, 80.0, 40.0));  // カーソル移動

// フレーム描画時
if tracker.is_full() {
    // 何かがビューポート全体を無効化 —— 全体を再描画
} else if let Some(region) = tracker.merged() {
    // render(region)
}
tracker.clear();
```

2 つの直交する状態：

- **full-damage フラグ**（`mark_full`）——ウィンドウリサイズやテーマ切替などでビューポート全体が無効化された時にセット。この間 `merged()` は `None` を返し、`is_full()` は `true`。個別の矩形は無視されます。
- **pending 矩形**（`add`）——`merged()` で 1 つのバウンディング矩形にマージ、もしくは `rects()` で生リストを取得し独自の合流戦略（保守的 / 積極的 / 閾値）を実装可能。

## 閾値戦略

ダーティ領域の合計が大きい場合、小さな矩形を多数描くより全体再描画のほうが安上がりです。`area_upper_bound()` は軽量な矩形面積合計（重複は二重計上）を返します:

```rust
if tracker.area_upper_bound() > viewport_area * 0.5 {
    // 部分描画を諦めて全体再描画
    tracker.mark_full();
}
```

## デモ

```bash
cargo run --example visualize -p damage-rects
```

インタラクティブなウィンドウ：

- クリックでダーティ矩形を追加
- `Space` でマージ領域を描画（緑の枠、その後クリア）
- `F` で full-damage フラグをトグル（黄色のティント）
- `C` でクリア

## インストール

```toml
[dependencies]
damage-rects = "0.1"
```

## 座標系

`DamageRect { x, y, width, height }`、`f32`。右/下は開区間（`contains_point` は右/下境界で `false` を返す）。座標の向きは呼び出し側に委ねられます——ライブラリは算術しか使いません。

<!-- ECOSYSTEM BEGIN (synced by claws/opensource/scripts/sync-ecosystem.py — edit ecosystem.toml, not this block) -->

## エコシステム

[metal-live-resize](https://crates.io/crates/metal-live-resize) · [coalesce-worker](https://crates.io/crates/coalesce-worker) · **damage-rects**

<!-- ECOSYSTEM END -->

## ライセンス

MIT —— [LICENSE](LICENSE) を参照。

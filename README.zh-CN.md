# damage-rects

[![Crates.io](https://img.shields.io/crates/v/damage-rects?style=flat-square&logo=rust)](https://crates.io/crates/damage-rects)
[![docs.rs](https://img.shields.io/docsrs/damage-rects?style=flat-square&logo=docs.rs)](https://docs.rs/damage-rects)
[![License](https://img.shields.io/crates/l/damage-rects?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.94-blue?style=flat-square&logo=rust)](Cargo.toml)
[![Downloads](https://img.shields.io/crates/d/damage-rects?style=flat-square)](https://crates.io/crates/damage-rects)

[English](README.md) | **简体中文** | [日本語](README.ja.md)

收集脏矩形并产出合并后的重绘区域，支持 GPU 画布的局部更新。零依赖、与 GPU 无耦合、不假设视口——可与任何 2D 渲染器（Skia、wgpu、softbuffer、原生 Metal）搭配。

> **同义词 / 你可能在搜索**：damage tracking、脏矩形跟踪、局部重绘（partial redraw）、失效区域（invalidation region）、重绘区域、区域合并、编辑器/终端增量重绘。

## 为什么

每一个交互式 GPU 渲染应用都要做选择：每次 tick 整个重绘，或者追踪哪些区域改变、只重绘这些。前者简单但浪费电，后者是编辑器、设计工具、终端和仪表盘实际在做的。

GUI 框架（`gpui`、`iced`、`slint`）都把 damage 跟踪藏在内部，从不作为可重用 crate 暴露。这个 crate 就是那个可重用实现。

## API

```rust
use damage_rects::{DamageRect, DamageTracker};

let mut tracker = DamageTracker::new();

// 状态更新时
tracker.add(DamageRect::new(10.0, 20.0, 100.0, 30.0)); // 某一行变了
tracker.add(DamageRect::new(50.0, 30.0, 80.0, 40.0));  // 光标移动

// 渲染帧时
if tracker.is_full() {
    // 有东西让整个视口失效 —— 整屏重绘
} else if let Some(region) = tracker.merged() {
    // render(region)
}
tracker.clear();
```

两个正交状态：

- **full-damage 标志**（`mark_full`）——窗口缩放、主题切换等让整个视口失效时设置。此时 `merged()` 返回 `None`，`is_full()` 返回 `true`，个别矩形会被忽略。
- **pending 矩形**（`add`）——通过 `merged()` 合并成一个 bounding rect，或通过 `rects()` 暴露出来让你实现自定义合并策略（保守 / 激进 / 阈值）。

## 阈值策略

当脏区域占总面积一大块时，整屏重绘反而比很多小矩形便宜。`area_upper_bound()` 给出一个便宜的（重叠会被重复计入）矩形面积之和：

```rust
if tracker.area_upper_bound() > viewport_area * 0.5 {
    // 放弃局部，整屏重绘
    tracker.mark_full();
}
```

## Demo

```bash
cargo run --example visualize -p damage-rects
```

交互式窗口：

- 点击添加脏矩形
- `Space` 渲染合并区域（绿色轮廓，然后清空）
- `F` 切换 full-damage 标志（黄色底色）
- `C` 清空

## 安装

```toml
[dependencies]
damage-rects = "0.1"
```

## 坐标系

`DamageRect { x, y, width, height }`，`f32`。右/下开半区间（`contains_point` 对右/下边界返回 `false`）。坐标方向由调用方决定——库内部仅做算术。

<!-- ECOSYSTEM BEGIN (generated — edit ecosystem.toml, not this block) -->

## 生态系统

[metal-live-resize](https://crates.io/crates/metal-live-resize) · [coalesce-worker](https://crates.io/crates/coalesce-worker) · **damage-rects**

<!-- ECOSYSTEM END -->

## 许可证

MIT —— 见 [LICENSE](LICENSE)。

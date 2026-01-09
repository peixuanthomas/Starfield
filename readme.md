# Starfield / 星空

A mesmerizing starfield visualization with random meteors in Rust.
一个用 Rust 编写的迷人星空可视化程序，带有随机流星效果。

## Features / 功能特性

- **Sparse Starfield** / **稀疏星空**
  - 180 static stars with subtle twinkling effects
  - 180 颗具有微妙闪烁效果的静态星星

- **Dynamic Meteors** / **动态流星**
  - Random meteors spawn with varying speeds and trajectories
  - 随机生成的流星，速度和轨迹多样化
  - Smooth fade-out tail effects
  - 平滑的渐隐尾迹效果

- **Performance Optimized** / **性能优化**
  - 60 FPS smooth animation
  - 60 FPS 平滑动画
  - Efficient pixel buffer rendering
  - 高效的像素缓冲区渲染

## Requirements / 需求

- Rust 1.56+
- Dependencies / 依赖：
  - `minifb` - Window and graphics rendering
  - `rand` - Random number generation

## Building / 构建

```bash
cargo build --release
```

## Running / 运行

```bash
cargo run --release
```

Press **ESC** to quit / 按 **ESC** 键退出

## Configuration / 配置

You can adjust these constants in `src/main.rs`:
你可以在 `src/main.rs` 中调整这些常量：

- `WIDTH` / `HEIGHT` - Window dimensions / 窗口尺寸
- `STAR_COUNT` - Number of background stars / 背景星星数量
- `METEOR_CHANCE` - Probability of meteor spawning each frame / 每帧流星生成概率

## How it works / 工作原理

1. **Stars** render with subtle twinkling and very slight radial drift
   - **星星** 以微妙的闪烁和极微弱的径向漂移渲染

2. **Meteors** spawn randomly with trajectories and fade over time
   - **流星** 随机生成，具有轨迹并随时间渐隐

3. Rendered at 60 FPS for smooth animation
   - 以 60 FPS 渲染，实现平滑动画

## Author / 作者

Created with Rust and minifb
用 Rust 和 minifb 创建
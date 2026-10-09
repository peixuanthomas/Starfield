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

- Current stable Rust / 当前稳定版 Rust
- Dependencies / 依赖：
  - `minifb` - Optional window rendering / 可选的窗口渲染
  - `rand` - Shared scene simulation / 共享场景模拟
  - `crossterm` - Cross-platform terminal input and screen handling / 跨平台终端操作
  - `base64`, `flate2` - Lossless terminal image transport / 无损终端图像传输

## Building / 构建

```bash
cargo build --release
```

## Running / 运行

```bash
cargo run --release
```

This still opens the original graphical window. Press **ESC** to quit.
默认仍打开原有图形窗口，按 **ESC** 退出。

### Terminal rendering / 终端渲染

```bash
cargo run --release -- --terminal
```

No graphical window or display server is opened in terminal mode. Both modes use
the **same 1200×800 pixel buffer, star/meteor simulation, colors and 60 Hz update rate**.
The image fits inside the terminal, centered with its original 3:2 aspect ratio
(to the nearest character cell). Resizing does not reset the scene. When the terminal
does not report cell pixel dimensions, a 1:2 cell aspect ratio is assumed.

终端模式不创建图形窗口，也不连接显示服务器。两种模式共用 **1200×800 像素缓冲区、
星点与流星模拟、颜色和 60 Hz 更新逻辑**。画面居中并保持原有 3:2 比例（精度受字符
尺寸影响），调整终端大小不会重置星空。无法读取字符像素尺寸时，按宽高比 1:2 估算。

| Mode / 模式 | Output / 显示效果 |
| --- | --- |
| `--terminal` or `--terminal=auto` | Kitty graphics in recognized Kitty/Ghostty sessions; ANSI otherwise. / 识别 Kitty、Ghostty 后使用图像协议，其余使用 ANSI。 |
| `--terminal=kitty` | Losslessly sends the original RGB frame to a terminal supporting the Kitty graphics protocol. / 将原始 RGB 画面无损传输给支持 Kitty 图像协议的终端。 |
| `--terminal=ansi` | True-color Unicode `▀` half blocks, two vertical samples per cell. / 使用真彩色半块字符，每个字符上下各显示一个采样点。 |

**For the same pixel image as the window, use `--terminal=kitty` in a compatible
terminal.** The original pixels are transmitted without color quantization;
terminal scaling can still change their displayed size. ANSI preserves the scene
and motion, but **cannot be pixel-identical** at character-grid resolution. It keeps
the brightest pixel in each sample region so isolated stars and meteor trails do
not disappear during downsampling. Separate runs start with different random scenes.

**要显示与窗口相同的像素画面，请在兼容终端使用 `--terminal=kitty`。** 原始像素无损
传输，最终显示尺寸由终端缩放决定。ANSI 模式保留场景与运动，但字符分辨率**无法做到
逐像素相同**；缩小时保留每个区域最亮的像素，避免细小星点和流星尾迹消失。每次独立
启动会随机生成不同星空。

Kitty mode requires a terminal with the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
enabled. WezTerm users can enable `config.enable_kitty_graphics = true` and explicitly
select `--terminal=kitty`. Auto mode conservatively selects ANSI under tmux/screen;
run directly in the terminal for image output. If image commands are unsupported
or filtered (a blank display), quit and use `--terminal=ansi`. ANSI mode requires
Unicode half-block glyphs and 24-bit color; appearance depends on the terminal font.

Kitty 模式需要终端开启 Kitty 图像协议。WezTerm 可设置
`config.enable_kitty_graphics = true`，然后显式选择 `--terminal=kitty`。
tmux/screen 中自动模式采用 ANSI；图像输出建议直接在终端运行。如果协议不支持或被
过滤而出现黑屏，退出后使用 `--terminal=ansi`。ANSI 模式需要 Unicode 半块字符及
24 位颜色支持，外观会受到终端字体影响。

Press **ESC**, **q**, or **Ctrl-C** to quit. The cursor, colors, line wrapping and
normal screen are restored on exit, I/O errors and panic unwinding. Input and output
must both be terminals; redirecting the animation to a file is rejected.

按 **ESC**、**q** 或 **Ctrl-C** 退出。正常退出、I/O 错误及 panic 展开时恢复光标、颜色、
换行模式和原终端屏幕。输入与输出都必须连接终端，不支持把动画重定向到文件。

### Terminal-only build / 纯终端构建

For a server/SSH session without X11, Wayland or other window-system development libraries:
没有 X11、Wayland 等图形开发库的服务器或 SSH 环境可使用：

```bash
cargo run --release --no-default-features -- --terminal
# Full RGB output / 完整 RGB 图像输出
cargo run --release --no-default-features -- --terminal=kitty
```

This omits `minifb` entirely. The remote machine does not need a desktop; image
support belongs to the terminal on your local machine. Actual presentation frame
rate depends on terminal and connection speed. `--help` lists available modes.

这会完全排除 `minifb`，远程主机不需要桌面环境；图像协议由本地终端提供支持。实际
显示帧率取决于终端与连接速度。使用 `--help` 查看选项。

## Configuration / 配置

You can adjust these constants in `src/scene.rs`:
你可以在 `src/scene.rs` 中调整这些常量：

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

## Validation / 验证

```bash
cargo fmt --check
cargo test --release
cargo test --release --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
```

Tests cover the existing long-running star simulation, lossless RGB decoding of
chunked terminal frames, sparse-star downsampling, resizing geometry, and quit keys.
测试覆盖原有长期运行模拟、分块终端图像的 RGB 无损解码、稀疏星点缩采样、缩放布局与退出按键。

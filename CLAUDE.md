# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

ffmpegConvert 是一个 **仅限 Windows** 的 Rust 单文件命令行工具，用于调用外部 `ffmpeg.exe` 对视频**批量转码**（H265 / AV1，压缩体积）。

使用模型是 **拖拽**：用户把一个或多个视频文件 / 文件夹拖到 `ffmpegConvert.exe` 图标上，这些路径作为命令行参数传入；程序递归收集视频文件，让用户选择转码预设序号，然后逐个转码。转码输出与原视频同目录。

## 常用命令

```powershell
cargo build --release        # 编译发布版，产物 target/release/ffmpegConvert.exe
cargo build                  # 调试版
cargo run -- "C:\video.mp4"  # 本地运行（参数即拖拽的路径，可多个；运行时需 ffmpeg.exe 可用）
cargo fmt                    # 格式化
cargo clippy -- -D warnings  # 静态检查
```

- **运行时依赖**：`ffmpeg.exe` 必须位于系统 PATH 或与本程序同目录（这不是 cargo 依赖，是运行期外部进程）。
- **无测试套件**：当前没有 `#[cfg(test)]` 模块，也没有 `tests/` 目录。
- `build.rs` 通过 `winres` 把 `icon.ico` 和版本信息嵌入 exe，因此编译仅支持 Windows 目标。

## 架构（大图景）

全部逻辑在 `src/main.rs`（约 690 行），没有 lib，没有子模块。核心是两段：

1. **`main()`** — 解析拖入的路径 → 递归收集视频文件（按扩展名白名单 `is_video_file` / `find_video_files`）→ 自然排序（`natural-sort-rs`）→ 让用户选预设序号 → 对每个文件构造输出路径并调用 `transcode_with_progress`。负数序号表示转码完成后调用 `shutdown.exe -s -t 30` 关机。

2. **`transcode_with_progress()`** — 单文件转码核心：
   - **串行化**：开始前用 `check_ffmpeg_running()`（调 `tasklist`）轮询，若已有其他 `ffmpeg.exe` 在跑就等待，保证全机只有一个 ffmpeg 实例在编码。
   - **进度解析**：`spawn` 出 `ffmpeg.exe -hide_banner -i <input> <预设参数> -y <output>`，捕获 **stderr** 并逐字节读取（按 `\r`/`\n` 切行），用 `parse_total_duration`（`Duration:` 行）和 `parse_progress`（`time=` / `speed=` 字段）计算百分比和 ETA，在同一行刷新进度，并用 `SetConsoleTitleW` 把进度写进控制台标题。
   - **完成后**：打印并写入体积对比日志（输入 → 输出，缩减百分比，带 ANSI 颜色），日志写到与 exe 同名的 `.log` 文件。

辅助函数：`parse_time_to_duration` / `format_duration`（时长解析与格式化）、`Color` + `color_text`（ANSI 着色）、`set_console_title`。

## 转码预设系统（最常修改的部分）

转码参数是本工具的"产品核心"。预设有两个来源：

1. **硬编码** 在 `main.rs` 的 `convert_params` 向量里（`ConvertParameter { params, subfix, description }`）。这是历史会话反复调优的对象（libx265 / hevc_amf 的 CQP 与 QVBR / libsvtav1 动画与电影档 / libaom-av1）。
2. **旁侧 `.txt` 覆盖**：`load_params_from_sidecar()` 会读取与 exe 同名、扩展名为 `.txt` 的文件追加预设。每行格式 `参数 # 输出后缀 # 说明`，`//` 或 `#` 开头的行跳过，参数部分必须含 `-`。这些字符串通过 `Box::leak` 泄漏为 `'static` 以兼容 `ConvertParameter<'static>`（CLI 短生命周期，可接受）。

> 注意：`README.md` 里列出的示例命令行参数可能与 `main.rs` 中的硬编码预设**不同步**（README 是旧版）。改预设时以 `main.rs` 为准，必要时同步更新 README。

## 关键约定与陷阱

- **Windows 专属 API**：`SetConsoleTitleW`（winapi/wincon）、`OsStrExt::encode_wide`、`tasklist`、`shutdown.exe`、ANSI 转义序列。代码不可移植到其他平台。
- **`[4..]` 切片剥离 verbatim 前缀**：`path.canonicalize()` 在 Windows 返回 `\\?\C:\...` 形式，代码用 `path_str[4..]` 去掉前 4 个字符的 `\\?\`（见 `main.rs` 约 245、256 行）。这依赖该前缀恒为 4 字符，是脆弱点，改动路径处理时要小心。
- **输出文件命名**：去掉 stem 中的 `_H264/_h264/_H265/_h265` 后追加预设的 `subfix`（如 `_H265`/`_AV1`）再加 `.mp4`。
- **防重复转码**：收集文件时会 `retain` 过滤掉 stem 以 `_h265` 或 `_av1`（小写比较）结尾的文件，避免把自己上次的输出当输入再转一遍。
- **编码风格**：遵循全局 Rust 规则（`edition = "2024"`、`let` 默认不可变、`&str`/`&[T]` 传参）。本项目当前用 `expect`/`unwrap` 较多（CLI 工具，失败即退出），新增代码应优先 `Result` + `?`。

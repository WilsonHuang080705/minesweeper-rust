# 扫雷游戏 (Rust 终端版)

## 介绍
本项目是一个基于 Rust 语言开发的终端版扫雷游戏，使用 `tui-rs` 进行 UI 渲染，`crossterm` 处理键盘输入，`rand` 生成随机地雷位置。

## 特性
- 终端界面美观，支持彩色渲染
- 三种难度模式：
  - 初级 (8x8, 10 雷)
  - 中级 (16x16, 40 雷)
  - 高级 (24x20, 99 雷)
- 支持键盘操作：
  - 方向键移动光标
  - 空格键翻开方格
  - `f` 键插/取消插旗
  - `r` 重新开始游戏
  - `q` 退出游戏
- 自动展开无雷区域
- 计时功能，显示剩余旗帜数

## 依赖
请确保您的环境已安装 Rust，并包含以下依赖：
- `crossterm`
- `tui`
- `rand`

## 安装和运行

1. 克隆本仓库
```sh
git clone https://github.com/WilsonHuang080705/minesweeper-rust.git
cd minesweeper-rust
```

2. 编译并运行
```sh
cargo run --release
```

## 许可证
本项目使用 [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) 许可协议。


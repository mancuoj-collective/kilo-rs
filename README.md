# kilo-rs

用**现代 Rust** 重写 [kilo](https://github.com/antirez/kilo) 文本编辑器——一个项目驱动的学习项目，
从零开始，一课一个小胜利。

## 课程

课程是一套可离线阅读、也可部署到 GitHub Pages 的 HTML 文档，位于 [`docs/`](docs/index.html)：

| #   | 课程                                                      | 主题                            |
| --- | --------------------------------------------------------- | ------------------------------- |
| 1   | [接管终端](docs/lessons/0001-raw-mode-and-events.html)    | raw mode、事件循环、`Drop` 守卫 |
| 2   | [干净的画布](docs/lessons/0002-a-clean-canvas.html)       | 备用屏幕、panic hook、画一帧    |
| 3   | [打开一个文件](docs/lessons/0003-open-a-file.html)        | 命令行参数、读文件、渲染        |
| 4   | [第一次重构](docs/lessons/0004-first-refactor.html)       | 拆出 `tui.rs` + `lib.rs`        |
| 5   | [光标与滚动](docs/lessons/0005-cursor-and-scrolling.html) | 光标、视口、单元测试            |
| 6   | [字符 ≠ 列](docs/lessons/0006-chars-vs-columns.html)      | tab、宽字符、`Row`              |
| 7   | [编辑与保存](docs/lessons/0007-editing-and-saving.html)   | 插入、换行、退格、写回磁盘      |
| 8   | [拆出 document 与 ui](docs/lessons/0008-document-and-ui.html) | 按接缝拆分缓冲区与渲染      |
| 9   | [状态栏与消息栏](docs/lessons/0009-status-and-message-bar.html) | 反馈、另存为、退出保护      |
| 10  | [查找](docs/lessons/0010-find.html)                       | `Ctrl-F` 实时搜索、命中高亮     |
| 11  | [语法高亮](docs/lessons/0011-syntax-highlighting.html)     | 规则表 + 状态机、`syntax.rs`    |

参考文档：[现代 Rust 惯用法](docs/reference/modern-rust-idioms.html) ·
[架构与文件结构](docs/reference/architecture.html) ·
[终端基础与 crossterm API](docs/reference/crossterm-terminal-basics.html)。

**部署到 GitHub Pages**：把仓库 Settings → Pages 的 Source 设为 `main` 分支的 `/docs` 目录，
之后访问 `https://<用户名>.github.io/<仓库名>/` 即可。

## 本地运行

```bash
cargo run -- kilo.c   # 打开一个文件
cargo run             # 空 buffer + 欢迎语
cargo test            # 单元测试（不启动终端）
cargo clippy --all-targets -- -D warnings
```

## 项目结构

```text
src/
├── main.rs     # 薄外壳：初始化 + 调用库
├── lib.rs      # 逻辑入口：run()、事件循环、prompt
├── tui.rs      # 终端生命周期（raw mode / 备用屏幕 / panic hook）
├── editor.rs   # 编辑器状态：光标、视口、消息、查找
├── document.rs # 缓冲区（域模型）：行、增删、序列化、保存
├── row.rs      # 一行文本 + 字符/显示列换算
└── ui.rs       # 渲染：&Editor -> 终端（无状态）
```

目标结构与拆分时机见 [架构与文件结构](docs/reference/architecture.html)。

## 约定

- **代码用英文**（注释、运行时字符串），**课程讲解用中文**；
- `cargo clippy --all-targets -- -D warnings` 零告警，禁用 `unsafe`。

## 参考资料

- 原版教程：<https://viewsourcecode.org/snaptoken/kilo/index.html>
- 原版源码：<https://github.com/antirez/kilo>
- Rust 版对照：<https://www.flenker.blog/hecto/>
- 最佳实践：<https://github.com/apollographql/rust-best-practices>

## 许可

- 本项目自己的代码与课程内容：**MIT**（见 [`LICENSE`](LICENSE)）。
- 第三方组件各自遵循其原始许可：
  - `kilo.c` —— 原版 [antirez/kilo](https://github.com/antirez/kilo)。
  - 中文字体 **仓耳今楷02（TsangerJinKai02）**：个人使用免费，商用需向 [tsanger.cn](https://tsanger.cn) 授权；
    本站按课程用字子集为 woff2 自托管，仅用于本课程展示。
  - Rust 依赖（crossterm / color-eyre / unicode-width）：MIT 或 Apache-2.0。
  - 代码高亮 [Prism.js](https://prismjs.com/)：MIT。

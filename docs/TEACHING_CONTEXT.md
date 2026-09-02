# Kilo Rust 教学上下文

这份文档用于新 session 接续项目和教学，不是产品 README。

## 项目目标

在现有 `kilo.c` 的基础上，用 Rust 从零重写一个 Kilo 文本编辑器。教学依据
Snaptoken 的 Build Your Own Text Editor 教程，但不是逐行翻译 C，而是保留
功能递进并采用适合当前 Rust 的实现。

仓库当前包含：

- `kilo.c`：完整的 C 版本，约 1068 行，是行为参考。
- `README.md`：教程链接。
- `Cargo.toml`：Rust 2024 binary crate，已有 `anyhow` 和 `crossterm`。
- `justfile`：统一格式化、检查、测试、lint、运行和编译 C 版本。

## 已确定的技术基线

- 使用已有的 `crossterm = 0.29`，不引入第二套终端库。
- 应用层使用 `anyhow::Result`，在 I/O 边界用 `Context` 添加语义错误。
- 用 `Editor` 结构体代替 C 的全局 `editorConfig E`。
- 用 `Vec` 代替 `malloc/realloc/free`。
- 用 RAII/`Drop` 恢复终端，不使用 `atexit`。
- 不用 `extern crate`、正常路径不使用 `unwrap()`。
- 第一版文本先限定 ASCII，并计划使用 `Vec<u8>`；Unicode 光标和宽度另做扩展。
- 早期模块可以存在，但不要创建只有声明没有职责的空抽象；不提前引入泛型、trait
  或多 crate 架构。

目标模块边界：

```text
src/
├── main.rs       # 程序入口
├── app.rs        # Editor 状态、主循环、按键分发
├── terminal.rs   # raw mode、尺寸、光标和屏幕输出
├── key.rs        # 内部按键类型和 crossterm 转换
├── buffer.rs     # 文件内容和行集合
├── row.rs        # 单行编辑、Tab 渲染、坐标转换
├── render.rs     # 编辑区、状态栏、消息栏
├── prompt.rs     # 保存名称和搜索输入
├── search.rs     # 搜索状态
└── syntax.rs     # 语法高亮
```

模块按需要逐步填充，避免空文件脚手架。

## 教学方式

每一课必须按以下顺序提供：

1. 本课目标和可观察结果。
2. Snaptoken 教程章节和本地 `kilo.c` 函数/行号。
3. 相关 C 代码参考和逐段解释。
4. Rust 模块设计、所有权/借用和错误处理提示。
5. 逐步骤手写指引。
6. 本课所有相关文件的完整可运行参考代码。
7. 使用 `just` 的检查命令和手动验收清单。

用户通常会先手写，再把工作区交给 agent review 和优化。第一课用户明确表示
已有完整参考代码，因此跳过了 review；后续是否 review 以用户当前指示为准。

每课完成并检查通过后：

1. 写入 `docs/lessons/NN-*.md`。
2. 记录目标、C 对照、Rust 决策、命令和已知限制。
3. 只提交本课相关文件。
4. 给出 commit hash 和下一课任务。
5. 用户说“开始下一节”后才开始下一课。

## 课程路线

当前规划 18 个主课时：

1. 应用骨架、Raw Mode、Ctrl-Q
2. 终端尺寸和最小屏幕绘制
3. 按键内部模型和光标绘制
4. 光标移动与滚动基础
5. 打开文件和多行查看
6. Tab 渲染与字符/屏幕坐标
7. PageUp/PageDown/Home/End
8. 状态栏和消息栏
9. 行模型与插入普通字符
10. Enter、Backspace、行拆分合并
11. dirty 标志和保存
12. Save As 与退出确认
13. prompt 抽象
14. 增量搜索
15. 搜索方向、循环和取消恢复
16. 高亮数据模型、数字和字符串
17. 文件类型、关键字和单行注释
18. 多行注释、非打印字符和最终整理

## 第一课当前状态

第一课实现：

- `src/main.rs`：声明模块，创建并运行 `Editor`。
- `src/app.rs`：`Editor` 持有 `RawMode`，循环读取按键，Ctrl-Q 退出。
- `src/key.rs`：将 crossterm 事件转换为 `Key::Ctrl`、`Key::Char`、`Key::Other`。
- `src/terminal.rs`：进入 raw mode，清屏、隐藏光标，`Drop` 时恢复终端。
- `key.rs` 有一个 Ctrl-Q 映射单元测试。

第一课文档：`docs/lessons/01-raw-mode.md`。

第一课 commit：

```text
53dbb8f lesson 01: enter raw mode and handle ctrl-q
```

第一课检查已通过：

```bash
just fmt
just check
just test
just lint
```

交互式 `just run` 需要在真实终端手动验证，自动检查没有运行它以避免挂起。

## 当前下一步

第二课目标是使用 `crossterm` 获取终端尺寸，绘制空白编辑区域、波浪线和基础
光标位置。不要提前实现文件读取、行缓冲区或语法高亮。开始前仍需先给出：

- 对应 C 代码（`getWindowSize`、清屏/光标输出相关部分）。
- 详细手写步骤。
- 使用 `anyhow` 的完整参考代码。
- `just fmt/check/test/lint` 验收方式。

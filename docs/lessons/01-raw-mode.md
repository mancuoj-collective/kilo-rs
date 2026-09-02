# 第 1 课：Raw Mode 与 Ctrl-Q

## 目标

建立最小的 Rust 编辑器应用：进入终端 raw mode，读取键盘事件，按 Ctrl-Q
退出，并在退出时自动恢复终端状态。

本课暂不处理文件、光标、屏幕滚动和文本编辑。

## C 版本对照

参考 `kilo.c`：

- `enableRawMode` / `disableRawMode`：终端模式切换（约 127-152 行）
- `editorReadKey`：读取一个按键（约 155-203 行）
- `editorProcessKeypress`：处理 Ctrl-Q（约 951-1031 行）
- `main`：启动和主循环（约 1052-1068 行）

C 版本使用 `termios` 配置 raw mode，并用 `atexit` 注册清理函数。Rust 版本
使用已有的 `crossterm`，由 `RawMode` 的 `Drop` 实现替代 `atexit`。

## 模块

```text
src/
├── main.rs       # 应用入口
├── app.rs        # Editor 和主循环
├── key.rs        # crossterm 事件到内部 Key 的转换
└── terminal.rs   # raw mode 和终端清理
```

`Editor` 暂时只持有 `RawMode`。后续课程会在这里增加光标、缓冲区和编辑器
状态，而不把状态重新放回全局变量。

## 错误处理

应用层统一使用 `anyhow::Result`。终端初始化和键盘读取通过 `Context` 增加
操作语义，例如 `failed to enable raw mode`。`Drop` 无法返回错误，因此清理
阶段忽略终端恢复错误；启动阶段的错误仍然正常返回。

## 按键模型

`key.rs` 将底层事件转换为内部枚举：

- `Key::Ctrl(char)`：控制键，例如 Ctrl-Q
- `Key::Char(char)`：普通字符，为后续编辑保留
- `Key::Other`：当前未处理的终端事件

Ctrl-Q 不再依赖 C 中的 ASCII 数值 `0x11`，而是通过
`KeyModifiers::CONTROL` 和 `KeyCode::Char('q')` 表达。

## 检查

```bash
just fmt
just check
just test
just lint
just run
```

手动验收：程序启动后清屏并显示提示，普通按键不会退出，Ctrl-Q 退出，光标
重新显示，终端不残留 raw mode。单元测试覆盖 Ctrl-Q 到内部 `Key` 的转换。

## 当前限制

- 只处理 Ctrl-Q 和普通字符事件。
- 不读取命令行文件参数。
- 终端清理错误在 `Drop` 中被忽略，这是 Rust 清理接口的限制。
- 文本和 Unicode 策略尚未引入。

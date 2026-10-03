# AGENTS.md

用现代 Rust 重写 [kilo](https://github.com/antirez/kilo) 文本编辑器的**教学项目**，以「课」推进：
**学习者亲手敲代码，你负责讲解、出练习、答疑**。

## 教学约定

- **代码英文，讲解中文。** 注释（`//` `///` `//!`）、目录树注释、运行时字符串（`expect` / `panic!` /
  错误信息）一律英文；课程正文中文。中文术语有歧义（「字符」可指码点 / 字素 / 列），
  英文精确（code point / column / byte）。
- **现代写法**：采用现代惯用法与 best practice，不照抄 hecto / C 版直译；依据 Apollo《Rust Best Practices》。
- **每课能跑**：每一步都要 `cargo run` 能看到变化；先体验问题，再引入解法。
- **质量门**：`cargo clippy --all-targets -- -D warnings` 零告警、`cargo fmt --check` 干净、禁用 `unsafe`。
- **站点高亮**：代码块标 `class="language-rust"`（shell 用 `bash`、`Cargo.toml` 用 `toml`），
  并在 `</body>` 前引入 `<script src="../assets/prism.min.js" defer></script>`（`docs/index.html` 用 `assets/…`）。
  高亮由本地内置的 Prism（含 rust / bash / toml）完成，无需联网。
- **站点样式**：视觉遵循 **kami 1.17**——羊皮纸底 `#f5f4ed`、单一墨蓝 `#1B365D`、暖灰、衬线
  （字重锁 500，不用粗体 / 斜体）。遵循其「**减法规则**」：用**字号**做层级、**间距**做分组、
  **墨蓝**做强调；不要装饰性的短线、左竖条或强调边框——提示框只有「象牙底 + 圆角」，代码块也只有
  「象牙底 + 圆角（无边框）」，语义由纯文字墨蓝 `.tag` 标签承担。样式集中在 `docs/assets/style.css`，
  新页面只写正文结构。中文字体为 **仓耳今楷02**（个人使用免费），已按当前用字子集为 woff2 放在
  `docs/assets/fonts/`；**新增汉字后需重新子集化**，否则新字会回退到系统字体。
- **重构即搬家**：按接缝拆模块，不按行数；每一刀行为不变，不一边重构一边加功能。
- **让学习者敲**：不代劳。

## 范围

实现 kilo 的核心：打开 / 显示 / 移动 / 编辑 / 保存 / 查找。
暂不做：手写 `termios` / ANSI、rope / piece table、插件 / LSP、多 buffer、Vim 模态。

## 进度 → 下一步

已完成第 1–11 课，`src/` 与之一致（核心 + 语法高亮齐了）。**下一步：整体回顾**，
对照下方「与原版差距」清单决定取舍、收尾。

## 与原版 kilo（`kilo.c`）的差距（收尾时再看）

核心已对齐：接管终端 / 显示 / 移动 / 滚动 / 编辑 / 保存（含另存为）/ 未保存退出保护 / 查找 / 语法高亮。
以下尚未对齐：

- `PageUp` / `PageDown` 翻页；`Tab` 键插入 `\t`（原版 default 分支还会插入控制字节）。
- 控制字符渲染：原版把不可打印字节显示为反显的 `@`+c / `?`；我们原样输出且列宽按 0 算（有错位风险）。
- `Ctrl-H` = 退格（常见终端已映射成 Backspace，优先级低）。
- 状态栏文件名 `%.20s` 截断。
- 文案差异：退出警告、查找提示、另存为提示、欢迎语（原版居中）、版本号（原版 `0.0.1`）。
- 保存失败：原版显示消息后继续运行，我们 `?` 抛错退出。
- 查找的匹配对象：原版在 tab 展开后的 `render` 上搜，我们在原始字符上搜。
- `Row::rx_to_cx` 目前仅测试用（原版在查找里用到）。
- Rust 的字符字面量 `'x'` 未单独上色：Rust 规则只把 `"` 当字符串引号，避免把生命周期 `'a` 当字符串。

## 地图（按需查阅）

- 课程全貌与目录 → `docs/index.html`
- 拆分路线图、目标结构、模块接口 → `docs/reference/architecture.html`
- 现代写法速查（`let-else`、RAII、错误处理、字符串转换）→ `docs/reference/modern-rust-idioms.html`
- 终端 API 与常见坑 → `docs/reference/crossterm-terminal-basics.html`
- 项目总览与 GitHub Pages 部署 → `README.md`

`src/` 模块：`main` 薄外壳 · `lib` 入口（`run` / 事件循环 / `prompt`）· `tui` 终端生命周期 ·
`editor` 光标、视口、消息、查找 · `document` 缓冲区（域模型）· `row` 行文本与列宽 · `ui` 渲染。

## 学习者

rustlings 水平（懂基本语法、`enum`/`match`、`Result`、迭代器；正在补借用冲突、`Drop`、拆模块）。
偏好现代写法与多文件结构。会主动质疑代码（例如指出两处写法不一致、兜底多余、`as` 与 `expect` 的取舍）——
欢迎这种质疑，深入解释，不要简单肯定。

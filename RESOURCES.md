# kilo-rs 学习资源

## Knowledge

- [Apollo GraphQL · Rust Best Practices Handbook](https://github.com/apollographql/rust-best-practices) —
  **本课程「现代写法」的主要依据**。第 1 章惯用法、第 2 章 lint、第 4 章错误处理、第 8 章注释与文档。
  凡是「hecto 那样写，我们这样写」的地方，理由多半在这里。
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) — 官方 API 设计约定，命名与接口设计参考。
- [Hecto: Build Your Own Text Editor in Rust](https://www.flenker.blog/hecto/) — Philip Flenker 的 Rust 版
  kilo 教程，和我们的路线最接近。**每章都值得先读再写**。用它对照、卡住时看它的做法。
- [Build Your Own Text Editor（C 版，snaptoken）](https://viewsourcecode.org/snaptoken/kilo/index.html) —
  原版教程，本仓库 `kilo.c` 的来源。讲清「为什么这么做」的一手材料，终端部分尤其权威。
- [antirez/kilo 源码](https://github.com/antirez/kilo) — 一千行出头的 C 原版，短小完整，适合通读。
- [crossterm 0.29 文档](https://docs.rs/crossterm/0.29.0/crossterm/) — 终端控制 API 的权威来源。
  遇到不认识的函数先查这里。
- [The Rust Programming Language（官方书）](https://doc.rust-lang.org/book/) — 所有权、借用、模块、
  错误处理查阅。重点章节：第 4 章（所有权）、第 6 章（枚举/match）、第 10 章（生命周期）。
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/) — 语法速查，比官方书更适合「我就想看个例子」。
- [Rust 语言圣经（中文）](https://course.rs/) — 中文 Rust 教材，讲所有权和借用很细。

## Wisdom (Communities)

- [Rust Users Forum](https://users.rust-lang.org/) — 提问所有权/借用问题的最佳去处，回复质量高。
- [r/rust](https://reddit.com/r/rust) — 综合讨论，适合看别人怎么写真实项目。
- [Rust 中文社区 (rustcc)](https://rustcc.cn/) — 中文问答与讨论。
- [crossterm GitHub Issues](https://github.com/crossterm-rs/crossterm/issues) — 终端行为在不同平台上的
  坑，常有人已经踩过。

## Gaps

- 还没有针对「编辑器数据结构演进（`Vec<String>` → `Row` → rope）」的高级资料，等课程走到那里再补。

# 教学笔记

## 学习者偏好

- 语言：中文讲解，技术名词保留英文。
- 学习方式：**自己敲代码**，不要 agent 代劳；agent 负责讲解、出练习、答疑。
- 终端底层：接受用 `crossterm` 简化，不追求手写 `termios`。
- **写法要求：只用最新、最现代的 Rust 惯用法与 best practice，不要照抄 hecto / C 版 kilo 的直译。**
  遇到「教程那样写，但现代 Rust 不这样写」时，要主动用现代写法并说明理由。依据：
  Apollo《Rust Best Practices》、Rust API Guidelines。课程风格速查见
  `reference/modern-rust-idioms.html`。
- **架构要求：要现代 Rust 的多文件结构，不要单文件写完。** 目标结构与拆分时机见
  `reference/architecture.html`。原则：按「接缝 / 深模块」拆，不按行数；每刀是「行为不变的搬家」，
  先搬家、后加功能。
- **语言要求：代码里的一切（注释 `//` `///` `//!`、目录树注释，以及运行时字符串如
  `expect` / `panic!` / 错误信息）一律用英文**；课程讲解正文保持中文。
  理由：中文里「字符」等词有歧义，英文术语精确（code point / column / byte），且与 Rust 生态一致。

## 已确立的判断

- 水平：自述「刚写过 rustlings」。据此推断：基础语法、`struct`/`enum`、`match`、`Option`/`Result`、
  迭代器、模块基础都见过；但真实项目里的**可变借用冲突**、集合增删改、`Drop`/RAII
  可能还很生。详见 `learning-records/0001`。
- 课程组织成「单车换轮子」式的小步：每课一个可运行、可观察的胜利，优先体验问题再引入解法。

## 待办 / 后续钩子

- **第 4 课（下一步）：拆出 `tui.rs` + `lib.rs`**（行为不变的搬家），之后再上光标与滚动。
- 数据模型从 `Vec<String>` 演进成 `Document` + `Row`，这里是借用冲突集中爆发处，提前铺垫。
- 完整目标结构与拆分路线图见 `reference/architecture.html`。

# 教学笔记

## 学习者偏好

- 语言：中文讲解，技术名词保留英文。
- 学习方式：**自己敲代码**，不要 agent 代劳；agent 负责讲解、出练习、答疑。
- 终端底层：接受用 `crossterm` 简化，不追求手写 `termios`。
- **写法要求：只用最新、最现代的 Rust 惯用法与 best practice，不要照抄 hecto / C 版 kilo 的直译。**
  遇到「教程那样写，但现代 Rust 不这样写」时，要主动用现代写法并说明理由。依据：
  Apollo《Rust Best Practices》、Rust API Guidelines。课程风格速查见
  `reference/modern-rust-idioms.html`。

## 已确立的判断

- 水平：自述「刚写过 rustlings」。据此推断：基础语法、`struct`/`enum`、`match`、`Option`/`Result`、
  迭代器、模块基础都见过；但真实项目里的**可变借用冲突**、集合增删改、`Drop`/RAII
  可能还很生。详见 `learning-records/0001`。
- 课程组织成「单车换轮子」式的小步：每课一个可运行、可观察的胜利，优先体验问题再引入解法。

## 待办 / 后续钩子

- 第 2 课：`EnterAlternateScreen` + 画文件内容 + 请求窗口大小 + `Drop` 守卫自动恢复终端。
- 数据模型后期会从「Vec<String>」演进成 `Row` 结构，这是借用冲突集中爆发的地方，提前铺垫。

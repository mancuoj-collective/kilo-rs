#[derive(Clone, Copy)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

pub struct Editor {
    lines: Vec<String>,
    /// 光标位置，以字符计：`cx` 是列，`cy` 是行
    cx: usize,
    cy: usize,
    /// 视口左上角对应文件的 `(列, 行)`。
    coloff: usize,
    rowoff: usize,
}

impl Editor {
    #[must_use]
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            cx: 0,
            cy: 0,
            coloff: 0,
            rowoff: 0,
        }
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    #[must_use]
    pub fn cursor(&self) -> (usize, usize) {
        (self.cx, self.cy)
    }

    #[must_use]
    pub fn viewport(&self) -> (usize, usize) {
        (self.coloff, self.rowoff)
    }

    pub fn move_cursor(&mut self, direction: Move) {
        let numrows = self.lines.len();
        match direction {
            Move::Left => {
                if self.cx > 0 {
                    self.cx -= 1;
                } else if self.cy > 0 {
                    self.cy -= 1;
                    self.cx = self.current_line_len();
                }
            }
            Move::Right => {
                if self.cy < numrows {
                    if self.cx < self.current_line_len() {
                        self.cx += 1;
                    } else {
                        self.cy += 1;
                        self.cx = 0;
                    }
                }
            }
            Move::Up => {
                if self.cy > 0 {
                    self.cy -= 1;
                }
            }
            Move::Down => {
                if self.cy < numrows {
                    self.cy += 1;
                }
            }
            Move::Home => self.cx = 0,
            Move::End => self.cx = self.current_line_len(),
        }

        // 移到更短的行之后，把列夹回行尾
        self.cx = self.cx.min(self.current_line_len());
    }

    pub fn scroll(&mut self, rows: usize, cols: usize) {
        if self.cy < self.rowoff {
            self.rowoff = self.cy;
        }
        if self.cy >= self.rowoff + rows {
            self.rowoff = self.cy - rows + 1;
        }
        if self.cx < self.coloff {
            self.coloff = self.cx;
        }
        if self.cx >= self.coloff + cols {
            self.coloff = self.cx - cols + 1;
        }
    }

    fn line_len(&self, cy: usize) -> usize {
        self.lines.get(cy).map_or(0, |line| line.chars().count())
    }

    fn current_line_len(&self) -> usize {
        self.line_len(self.cy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> Editor {
        Editor::new(vec![String::from("hello"), String::from("world!!")])
    }

    #[test]
    fn right_moves_within_a_line() {
        let mut e = editor();
        e.move_cursor(Move::Right);
        assert_eq!(e.cursor(), (1, 0));
    }

    #[test]
    fn moving_up_clamps_column_to_line_end() {
        let mut e = editor();
        e.move_cursor(Move::End); // 第一行末尾：列 5
        e.move_cursor(Move::Down);
        assert_eq!(e.cursor(), (5, 1));
        e.move_cursor(Move::Up);
        assert_eq!(e.cursor(), (5, 0));
    }

    #[test]
    fn scroll_keeps_cursor_visible() {
        let mut e = Editor::new((0..10).map(|i| format!("line {i}")).collect());
        e.move_cursor(Move::Down);
        e.move_cursor(Move::Down);
        e.move_cursor(Move::Down); // cy = 3
        e.scroll(3, 20); // 视口只有 3 行高
        assert_eq!(e.viewport(), (0, 1)); // rowoff = 3 - 3 + 1
    }
}

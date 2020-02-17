use std::iter::{self, Iterator};

use unicode_width::UnicodeWidthStr;

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::symbols::line;
use tui::widgets::{Block, List, Text, Widget};

use super::app::Remind;

pub struct TodoList<'b> {
    block: Option<Block<'b>>,
    /// Items to be displayed
    item: &'b Remind,
    /// Index of the one selected
    selected: Option<usize>,
    /// Base style of the widget
    style: Style,
    /// Style used to render selected item
    highlight_style: Style,
    /// Symbol in front of the selected item (Shift all items to the right)
    highlight_symbol: Option<&'b str>,
}

impl<'b> TodoList<'b> {
    pub fn new(item: &'b Remind) -> TodoList<'b> {
        TodoList {
            block: None,
            item,
            selected: None,
            style: Default::default(),
            highlight_style: Default::default(),
            highlight_symbol: None,
        }
    }
    pub fn block(mut self, block: Block<'b>) -> TodoList<'b> {
        self.block = Some(block);
        self
    }

    pub fn highlight_symbol(mut self, highlight_symbol: &'b str) -> TodoList<'b> {
        self.highlight_symbol = Some(highlight_symbol);
        self
    }

    pub fn highlight_style(mut self, highlight_style: Style) -> TodoList<'b> {
        self.highlight_style = highlight_style;
        self
    }

    pub fn select(mut self, index: Option<usize>) -> TodoList<'b> {
        self.selected = index;
        self
    }
}

impl<'b> Widget for TodoList<'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let list_area = match self.block {
            Some(ref mut b) => b.inner(area),
            None => area,
        };

        let list_height = list_area.height as usize;

        // Use highlight_style only if something is selected
        let (selected, highlight_style) = match self.selected {
            Some(i) => (Some(i), self.highlight_style),
            None => (None, self.style),
        };
        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let blank_symbol = iter::repeat(" ")
            .take(highlight_symbol.width())
            .collect::<String>();
        // Make sure the list show the selected item
        let offset = if let Some(selected) = selected {
            if selected >= list_height {
                selected - list_height + 1
            } else {
                0
            }
        } else {
            0
        };

        // Render items
        let item = self
            .item
            .list
            .iter()
            .enumerate()
            .map(|(i, todo)| {
                let strike = if todo.completed {
                    Modifier::CROSSED_OUT
                } else {
                    Modifier::ITALIC
                };
                if let Some(s) = selected {
                    if i == s {
                        let style = Style::default()
                            .bg(highlight_style.bg)
                            .fg(highlight_style.fg)
                            .modifier(strike);
                        Text::styled(format!("{} {}", highlight_symbol, todo.as_str()), style)
                    } else {
                        let style = Style::default()
                            .bg(self.style.bg)
                            .fg(self.style.fg)
                            .modifier(strike);
                        Text::styled(format!("{} {}", blank_symbol, todo.as_str()), style)
                    }
                } else {
                    Text::styled(todo.as_str().to_string(), self.style)
                }
            })
            .skip(offset as usize);
        List::new(item)
            .block(self.block.unwrap_or_default())
            .style(self.style)
            .draw(area, buf);
    }
}

struct Wrapper {
    wrap: bool,
    rows: u16,
}

pub struct TabsWrapped<'a, T>
where
    T: AsRef<str> + 'a,
{
    /// A block to wrap this widget in if necessary
    block: Option<Block<'a>>,
    /// One title for each tab
    titles: &'a [T],
    /// Wraps the tab bar when the length of tab chars overflows
    /// witdth of enclosing `Block`.
    wrap: Wrapper,
    /// The index of the selected tabs
    selected: usize,
    /// The style used to draw the text
    style: Style,
    /// The style used to display the selected item
    highlight_style: Style,
    /// Tab divider
    divider: &'a str,
}

impl<'a, T> Default for TabsWrapped<'a, T>
where
    T: AsRef<str>,
{
    fn default() -> TabsWrapped<'a, T> {
        TabsWrapped {
            block: None,
            titles: &[],
            wrap: Wrapper { wrap: false, rows: 0, },
            selected: 0,
            style: Default::default(),
            highlight_style: Default::default(),
            divider: line::VERTICAL,
        }
    }
}

impl<'a, T> TabsWrapped<'a, T>
where
    T: AsRef<str>,
{
    pub fn block(mut self, block: Block<'a>) -> TabsWrapped<'a, T> {
        self.block = Some(block);
        self
    }

    pub fn titles(mut self, titles: &'a [T]) -> TabsWrapped<'a, T> {
        self.titles = titles;
        self
    }

    pub fn wrap(mut self, wrap: bool, rows: u16) -> TabsWrapped<'a, T> {
        self.wrap = Wrapper { wrap, rows, };
        self
    }

    pub fn select(mut self, selected: usize) -> TabsWrapped<'a, T> {
        self.selected = selected;
        self
    }

    pub fn style(mut self, style: Style) -> TabsWrapped<'a, T> {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> TabsWrapped<'a, T> {
        self.highlight_style = style;
        self
    }

    pub fn divider(mut self, divider: &'a str) -> TabsWrapped<'a, T> {
        self.divider = divider;
        self
    }
}

impl<'a, T> Widget for TabsWrapped<'a, T>
where
    T: AsRef<str>,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let overflow = {
            area.width as usize <= self.titles.iter()
                .enumerate()
                .map(|(i, s)| {
                    let space = if i % 2 == 0 {
                        3
                    } else {
                        2
                    };
                    s.as_ref().width() + space
                })
                .sum()
        };
        if self.wrap.wrap && overflow {
            let tabs_area = match self.block {
                Some(ref mut b) => {
                    b.draw(area, buf);
                    b.inner(area)
                }
                None => area,
            };

            if tabs_area.height < 1 {
                return;
            }

            self.background(tabs_area, buf, self.style.bg);

            let mut x = tabs_area.left();
            let mut y = tabs_area.top();
            let titles_length = self.titles.len();
            let divider_width = self.divider.width() as u16;
            let title_style_iter = self.titles.iter()
                .zip(self.titles.iter().skip(1))
                .enumerate()
                .map(|(i, t)| {
                    let lt = i + 1 == titles_length;
                    if i == self.selected {
                        (t, self.highlight_style, lt)
                    } else {
                        (t, self.style, lt)
                    }
                });
            for ((title, next_title), style, last_title) in title_style_iter {
                let title_len = title.as_ref().width() as u16 + 1;
                x += 1;

                if x + title_len >= tabs_area.right() {
                    y += 1;
                    x = tabs_area.left() + 1;
                }
                if y > self.wrap.rows {
                    break;
                }

                buf.set_string(x, y, title.as_ref(), style);
                x += title.as_ref().width() as u16 + 1;

                let has_overflow = x + next_title.as_ref().width() as u16 + 1 >= tabs_area.right();
                let last_wrap_row = y + 1 > self.wrap.rows && has_overflow;
                println!(
                    "title={}\nover={} last={}\narea={:?} x={} next={}\n",
                    title.as_ref(),
                    has_overflow,
                    last_wrap_row,
                    tabs_area,
                    x,
                    x + next_title.as_ref().width() as u16 + 1,
                );
                if x >= tabs_area.right() || last_title || has_overflow || last_wrap_row {
                    continue;
                } else {
                    buf.set_string(x, y, self.divider, self.style);
                    x += divider_width;
                }
            }
        } else {
            let tabs_area = match self.block {
                Some(ref mut b) => {
                    b.draw(area, buf);
                    b.inner(area)
                }
                None => area,
            };

            if tabs_area.height < 1 {
                return;
            }

            self.background(tabs_area, buf, self.style.bg);

            let mut x = tabs_area.left();
            let titles_length = self.titles.len();
            let divider_width = self.divider.width() as u16;
            for (title, style, last_title) in self.titles.iter().enumerate().map(|(i, t)| {
                let lt = i + 1 == titles_length;
                if i == self.selected {
                    (t, self.highlight_style, lt)
                } else {
                    (t, self.style, lt)
                }
            }) {
                x += 1;
                if x > tabs_area.right() {
                    break;
                } else {
                    buf.set_string(x, tabs_area.top(), title.as_ref(), style);
                    x += title.as_ref().width() as u16 + 1;
                    if x >= tabs_area.right() || last_title {
                        break;
                    } else {
                        buf.set_string(x, tabs_area.top(), self.divider, self.style);
                        x += divider_width;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tui::backend::TestBackend;
    use tui::buffer::Buffer;
    use tui::layout::Alignment;
    use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
    use tui::Terminal;

    #[test]
    fn paragraph_render_wrap() {
        let render = || {
            let backend = TestBackend::new(20, 10);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|mut f| {
                    let size = f.size();
                    let text = [
                        "123",
                        "678",
                        "123",
                        "78",
                        "1234",
                        "7890",
                        "0000",
                    ];
                    TabsWrapped::default()
                        .titles(&text)
                        .block(Block::default().borders(Borders::ALL))
                        .wrap(true, 2)
                        .render(&mut f, size);
                })
                .unwrap();
            terminal.backend().buffer().clone()
        };
        println!("{:?}", render());
        assert_eq!(
            render(),
            Buffer::with_lines(vec![
                   "┌──────────────────┐",
                   "│ 123 │ 789 │ 123  │",
                   "│ 78 │ 1234        │",
                   "│                  │",
                   "│                  │",
                   "│                  │",
                   "│                  │",
                   "│                  │",
                   "│                  │",
                   "└──────────────────┘",
            ])
        );
    }
}

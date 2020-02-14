use std::convert::AsRef;
use std::iter::{self, Iterator};

use unicode_width::UnicodeWidthStr;

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
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

    pub fn style(mut self, style: Style) -> TodoList<'b> {
        self.style = style;
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

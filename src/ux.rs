use std::io;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Marker, Paragraph, Row,
    SelectableList, Sparkline, Table, Tabs, Text, Widget,
};
use tui::{Frame, Terminal};

use super::app::{App, Todo, Note};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        Tabs::default()
            .block(Block::default().borders(Borders::ALL).title(app.title))
            .titles(&app.tabs.titles)
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);

        match app.tabs.index {
            0 => draw_first_tab(&mut f, &app, chunks[1]),
            1 => draw_second_tab(&mut f, &app, chunks[1]),
            _ => {}
        };
    })
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(100),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(area);
    draw_main_page(f, app, chunks[0]);
}

fn draw_main_page<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![ Constraint::Percentage(100), ])
        .direction(Direction::Horizontal)
        .split(area);

    Block::default()
        .borders(Borders::ALL)
        .title("Sticky Notes")
        .render(f, area);
    
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(25), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[0]);

    for todo in app.sticky_note.iter() {
        SelectableList::default()
            .block(Block::default().borders(Borders::ALL).title(todo.title))
            .items(&todo.to_list().collect::<Vec<_>>())
            .select(Some(app.sticky_note.selected))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .render(f, chunks[0]);
    }
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    
    let items = app.sticky_note.get_selected().unwrap();
    let text = items.list.iter().enumerate().map(convert_text).collect::<Vec<_>>();
    
    Paragraph::new(text.iter())
        .block(Block::default()
            .borders(Borders::ALL)
            .title(items.title)
            .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD))
        )
        .wrap(true)
        .render(f, area);
}

fn convert_text<'a>(pair: (usize, &'a Todo)) -> Text<'a> {
    let (idx, todo) = pair;
    if idx % 2 == 0 {
        Text::styled(todo.task, Style::default().fg(Color::Blue))
    } else {
        Text::styled(todo.task, Style::default().fg(Color::Green))
    }
}

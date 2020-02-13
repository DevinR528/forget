use std::io;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{
    Block, Borders, Paragraph, SelectableList, Sparkline, Tabs, Text, Widget,
};
use tui::{Frame, Terminal};

use super::app::{App, Todo};

const ADD_REMIND: &str = "Title of reminder: ";
const ADD_TODO: &str = "What do you want ToDo: ";
const ADD_CMD: &str = "Command to run: ";

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        Tabs::default()
            .block(Block::default().borders(Borders::ALL).title(&app.title))
            .titles(&app.tabs.titles)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Cyan))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);

        draw_app(&mut f, &app, chunks[1])
    })
}

fn draw_app<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100), Constraint::Percentage(25)].as_ref())
        .split(area);
    draw_main_page(f, app, chunks[0]);
}

fn draw_main_page<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .split(area);

    // Block::default()
    //     .borders(Borders::ALL)
    //     .title("Sticky Notes")
    //     .render(f, area);

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(65), Constraint::Percentage(45)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[0]);

    let todo = &app.sticky_note[app.tabs.index];

    SelectableList::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(&todo.title)
                .title_style(Style::default().bg(Color::LightBlue).fg(Color::Black)),
        )
        .items(&todo.to_list().collect::<Vec<_>>())
        .select(Some(app.sticky_note.selected))
        .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
        .highlight_symbol(">")
        .render(f, chunks[0]);

    draw_util_block(f, app, chunks[1])
}

fn draw_util_block<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    if app.new_reminder {
        let remind_title = &app.add_remind.title;
        SelectableList::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add Sticky Note")
                    .title_style(Style::default().bg(Color::LightBlue).fg(Color::Black)),
            )
            .items(&[&format!("{}{}", ADD_REMIND, remind_title)])
            .select(Some(0))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol("*")
            .render(f, area);
    } else if app.new_todo {
        let task = &app.add_todo.task;
        let cmd = &app.add_todo.cmd;
        SelectableList::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add ToDo Item")
                    .title_style(Style::default().bg(Color::LightBlue).fg(Color::Black)),
            )
            .items(&[
                    &format!("{}{}", ADD_TODO, task),
                    &format!("{}{}", ADD_CMD, cmd),
                ])
            .select(Some(0))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol("*")
            .render(f, area);
    } else {
        let note = &app.sticky_note[app.tabs.index].note;
        let text = Text::styled(note, Style::default().fg(Color::Red).modifier(Modifier::BOLD));
        Paragraph::new(vec![text].iter())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add ToDo Item")
                    .title_style(Style::default().bg(Color::LightBlue).fg(Color::Black)),
            )
            .render(f, area);
    }
}

fn convert_text<'a>(pair: (usize, &'a Todo)) -> Text<'a> {
    let (idx, todo) = pair;
    if idx % 2 == 0 {
        Text::styled(&todo.task, Style::default().fg(Color::Blue))
    } else {
        Text::styled(&todo.task, Style::default().fg(Color::Green))
    }
}

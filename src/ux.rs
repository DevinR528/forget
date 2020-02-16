use std::io;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Tabs, Text, Widget};
use tui::{Frame, Terminal};

use super::app::{App, Remind};
use super::widget::TodoList;

const ADD_REMIND: &str = "Title of Sticky Note";
const ADD_TODO: &str = "What do you want Todo";
const ADD_CMD: &str = "Command to run";

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        // if all tabs fit on one row or split and draw two
        // if chunks[0].width > app.tabs.titles.iter().map(|s| (s.chars().count() + 2) as u16).sum() {
        Tabs::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(&app.title)
                    .title_style(
                        Style::default()
                            .fg(app.config.app_colors.titles.fg.into())
                            .modifier(app.config.app_colors.titles.modifier.into()),
                    ),
            )
            .titles(&app.tabs.titles)
            .style(Style::default().fg(app.config.app_colors.normal.fg.into()))
            .highlight_style(
                Style::default()
                    .fg(app.config.app_colors.tabs.fg.into())
                    .modifier(app.config.app_colors.tabs.modifier.into()),
            )
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);

        draw_app(&mut f, app, chunks[1])
    })
}

fn draw_app<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100), Constraint::Percentage(25)].as_ref())
        .split(area);
    draw_main_page(f, app, chunks[0]);
}

fn draw_main_page<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .split(area);

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(65), Constraint::Percentage(45)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[0]);

    let (todo, selected) = if let Some(todo) = app.sticky_note.items.get(app.tabs.index) {
        (todo.clone(), todo.list.selected)
    } else {
        (Remind::default(), 0)
    };

    TodoList::new(&todo)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(&todo.title)
                .title_style(
                    Style::default()
                        .bg(app.config.app_colors.titles.bg.into())
                        .fg(app.config.app_colors.titles.fg.into())
                        .modifier(app.config.app_colors.titles.modifier.into()),
                ),
        )
        .select(Some(selected))
        .highlight_style(
            Style::default()
                .fg(app.config.app_colors.highlight.fg.into())
                .bg(app.config.app_colors.highlight.bg.into())
                .modifier(app.config.app_colors.highlight.modifier.into()),
        )
        .highlight_symbol(&app.config.highlight_string)
        .render(f, chunks[0]);

    draw_util_block(f, app, chunks[1])
}

fn draw_util_block<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    if app.new_reminder {
        let remind_title = &app.add_remind.title;
        let highlight_style = Style::default().fg(Color::Yellow);

        Paragraph::new(
            vec![Text::styled(
                remind_title,
                Style::default().fg(Color::Green),
            )]
            .iter(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(highlight_style)
                .title(ADD_REMIND)
                .title_style(
                    Style::default()
                        .bg(Color::Black)
                        .fg(Color::Yellow)
                        .modifier(Modifier::BOLD),
                ),
        )
        .wrap(true)
        .render(f, area);
    } else if app.new_todo {
        let task = &app.add_todo.task;
        let cmd = &app.add_todo.cmd;
        let question = app.add_todo.question_index;

        let highlight_style = Style::default().fg(Color::Yellow);
        let normal_style = Style::default().fg(Color::White);

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .direction(Direction::Vertical)
            .split(area);

        Paragraph::new(vec![Text::styled(task, Style::default().fg(Color::Green))].iter())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if question == 0 {
                        highlight_style
                    } else {
                        normal_style
                    })
                    .title(ADD_TODO)
                    .title_style(
                        Style::default()
                            .bg(Color::Black)
                            .fg(if question == 0 {
                                highlight_style.fg
                            } else {
                                normal_style.fg
                            })
                            .modifier(Modifier::BOLD),
                    ),
            )
            .wrap(true)
            .render(f, chunks[0]);

        Paragraph::new(vec![Text::styled(cmd, Style::default().fg(Color::Green))].iter())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if question == 1 {
                        highlight_style
                    } else {
                        normal_style
                    })
                    .title(ADD_CMD)
                    .title_style(
                        Style::default()
                            .bg(Color::Black)
                            .fg(if question == 0 {
                                highlight_style.fg
                            } else {
                                normal_style.fg
                            })
                            .modifier(Modifier::BOLD),
                    ),
            )
            .wrap(true)
            .render(f, chunks[1]);
    } else {
        let note = &app.sticky_note.items.get(app.tabs.index).map(|n| n.note.clone()).unwrap_or_default();
        let text = Text::styled(note, Style::default().fg(Color::Green));
        Paragraph::new(vec![text].iter())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(if app.new_note {
                        "Add To Notes"
                    } else {
                        "Notes"
                    })
                    .title_style(Style::default().bg(Color::LightBlue).fg(Color::Black)),
            )
            .wrap(true)
            .render(f, area);
    }
}

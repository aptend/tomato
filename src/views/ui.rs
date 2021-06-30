use super::widgets::Countdown;
use crate::models::{ActiveBlock, App, InputContext, TabType};

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use tui::Frame;

use chrono::TimeZone;
use unicode_width::UnicodeWidthStr;

const COUNTDOWN_SIZES: &[(u16, u16, u16, u16)] = &[
    (7, 41, 0, 0),
    (13, 50, 7, 41),
    (17, 70, 9, 51),
    (21, 90, 11, 71),
    (25, 110, 13, 81),
    (u16::MAX, u16::MAX, 15, 91),
];

fn thick_border_or_not(app: &App, target_block: ActiveBlock) -> BorderType {
    if app.active_block() == target_block {
        BorderType::Thick
    } else {
        BorderType::Plain
    }
}

/// helper function to create a centered rect using up
/// certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn draw_app<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
        .split(f.size());

    draw_navitabs(f, app, chunks[0]);
    match app.tabs.tab_type() {
        TabType::Inventory => draw_inventory_tab(f, app, chunks[1]),
        TabType::Tomato => draw_tomato_tab(f, app, chunks[1]),
        TabType::Statistics => draw_statistic_tab(f, app, chunks[1]),
    };

    if app.input.is_active() {
        draw_input(f, app, f.size());
    }

    if let Some(msg) = &app.notify {
        draw_popup(f, msg, f.size());
    }
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let mut area = centered_rect(50, 90, area);
    let pad = area.height.saturating_sub(3) / 2;
    if pad == 0 {
        return;
    }
    area.height = 3;
    area.y += pad;

    let paragraph = Paragraph::new(app.input.content()).block(
        Block::default()
            .title(match app.input.unwrap_cxt() {
                InputContext::NewInventory(_) => "New inventory entry",
                InputContext::NewTask(_) => "New task",
                InputContext::EditInventory(_) => "Edit inventory entry",
                InputContext::EditTask(_) => "Edit task",
            })
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .borders(Borders::all()),
    );
    f.render_widget(paragraph, area);
    f.set_cursor(area.x + app.input.content().width() as u16 + 1, area.y + 1);
}

fn draw_popup<B: Backend>(f: &mut Frame<B>, msg: &str, area: Rect) {
    let area = centered_rect(60, 20, area);
    f.render_widget(Clear, area);
    let paragraph = Paragraph::new(msg).alignment(Alignment::Center).block(
        Block::default()
            .title("Notification")
            .border_type(BorderType::Double)
            .borders(Borders::all()),
    );
    f.render_widget(paragraph, area);
}

fn draw_navitabs<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(t, Style::default().fg(Color::Green))))
        .collect();

    let border_type = thick_border_or_not(app, ActiveBlock::Navitab);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(border_type),
        )
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.select);
    f.render_widget(tabs, area);
}

fn draw_inventory_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)].as_ref())
        .split(area);

    draw_inventory_list(f, app, chunks[0]);
    draw_task_list(f, app, chunks[1]);
}

fn draw_inventory_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let mut state = ListState::default();
    state.select(app.inventory.inventory_selected);
    let items: Vec<_> = app
        .inventory
        .inventory_list
        .iter()
        .map(|i| {
            let item = Spans::from(vec![
                Span::styled("●", Style::default().fg(i.color.into())),
                Span::raw(" "),
                Span::raw(&i.name),
            ]);
            ListItem::new(item)
        })
        .collect();

    let border_type = thick_border_or_not(app, ActiveBlock::InventoryList);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Inventory")
                .border_type(border_type),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("[d] ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_task_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if app.inventory.inventory_selected.is_none() {
        return;
    }
    let task_idx = app.inventory.inventory_selected.unwrap();
    let mut state = ListState::default();
    state.select(app.inventory.task_selected[task_idx]);

    // right alignment manually & center sep line
    let width = (area.width - 2) as usize; //  2 boderlines
    let sep_percent = 95;
    let sep_left_pad = width * ((100 - sep_percent) / 2) / 100;
    let sep_width = width * sep_percent / 100;
    let mut sep_line = " ".repeat(sep_left_pad);
    for _ in 0..sep_width {
        sep_line.push('-');
    }

    let task_last_idx = app.inventory.tasks_list[task_idx].len();

    let items: Vec<_> = app.inventory.tasks_list[task_idx]
        .iter()
        .enumerate()
        .map(|(idx, t)| {
            let mut line = t.name.clone();
            let date = chrono::Local
                .timestamp(t.create_at, 0)
                .format("%Y-%-m-%-d")
                .to_string();
            let padding = " ".repeat(width.saturating_sub(date.width() + line.width()));
            line.push_str(&padding);
            line.push_str(&date);

            let mut list_item = vec![
                Spans::from(line),
                Spans::from(format!("🍅 {} minutes", t.spent_minutes)),
            ];

            if idx < task_last_idx {
                list_item.push(Spans::from(Span::styled(
                    &sep_line,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            ListItem::new(list_item)
        })
        .collect();

    let border_type = thick_border_or_not(app, ActiveBlock::TaskList);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Task")
                .border_type(border_type),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_statistic_tab<B: Backend>(f: &mut Frame<B>, _app: &App, area: Rect) {
    f.render_widget(
        Paragraph::new("there are a lot of graphes"),
        Rect::new(area.x, area.bottom() - 1, area.width, 1),
    );
}

fn draw_tomato_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let countdown_area = draw_countdown(f, app, area);

    // debug info
    f.render_widget(
        Paragraph::new(format!(
            "status: {}x{}, {}x{}",
            countdown_area.width,
            countdown_area.height,
            f.size().width,
            f.size().height,
        )),
        Rect::new(area.x, area.bottom() - 1, area.width, 1),
    );
}

fn draw_countdown<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) -> Rect {
    // r: resize  w: widget
    let (wh, ww) = COUNTDOWN_SIZES
        .iter()
        .find(|(rh, rw, ..)| area.width < *rw || area.height < *rh)
        .map(|item| (item.2, item.3))
        .unwrap();

    if wh == 0 {
        f.render_widget(Paragraph::new("Too small to display a countdown"), area);
        return area;
    }

    // place Countdown in center
    let count_area = Rect::new(
        area.x + (area.width - ww) / 2,
        area.y + (area.height - wh) / 2,
        ww,
        wh,
    );

    let (m, s) = app.tomato.min_and_sec();
    f.render_widget(
        Countdown::default()
            .minutes(m)
            .seconds(s)
            .skin('〇')
            .digit_style(app.tomato.fg_style()),
        count_area,
    );

    if let Some((iidx, tidx)) = app.tomato.where_idx() {
        let mut task_info = app.inventory.inventory_list[iidx].name.clone() + " · ";
        task_info.push_str(&app.inventory.tasks_list[iidx][tidx].name);

        let up_offset_y = (count_area.y - area.y).min(2);
        let info_area = Rect {
            x: area.x,
            y: count_area.y - up_offset_y,
            width: area.width,
            height: 1,
        };

        f.render_widget(
            Paragraph::new(task_info).alignment(Alignment::Center),
            info_area,
        );
    }

    count_area
}

use super::widgets::Countdown;
use crate::models::navitab_model::TabType;
use crate::models::ActiveBlock;
use crate::models::App;

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use tui::Frame;

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

    if let Some(msg) = &app.notify {
        draw_popup(f, msg, f.size());
    }
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
                Span::styled("●", Style::default().fg(i.color)),
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
            let padding = " ".repeat(width.saturating_sub(t.crate_date.width() + line.width()));
            line.push_str(&padding);
            line.push_str(&t.crate_date);

            let mut list_item = vec![
                Spans::from(line),
                Spans::from(format!("🍅 {} minutes", t.tomato_minutes)),
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

    let border_type = thick_border_or_not(app, ActiveBlock::InventoryTaskList);

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

    if let Some((iidx, tidx)) = app.ongoing_tomato_idx {
        let mut task_info = app.inventory.inventory_list[iidx].name.clone() + " · ";
        task_info.push_str(&app.inventory.tasks_list[iidx][tidx].name);

        let up_offset_y = (count_area.y - area.y).min(2);
        let info_area = Rect {
            x: area.x,
            y: count_area.y - up_offset_y,
            width: area.width,
            height: 1,
        };
    
        f.render_widget(Paragraph::new(task_info).alignment(Alignment::Center), info_area);
    }

    
    count_area
}

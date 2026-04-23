use crate::app::App;
use crate::theme;
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans: Vec<Span> = Vec::new();

    if app.ui.connected {
        spans.push(Span::styled(" ● ", theme::ok_style()));
        if let Some(name) = &app.ui.device_name {
            spans.push(Span::raw(format!("{} ", name)));
        } else {
            spans.push(Span::raw("connecté "));
        }
    } else {
        spans.push(Span::styled(" ○ ", theme::err_style()));
        spans.push(Span::raw("déconnecté "));
    }

    if let Some(rssi) = app.ui.last_rssi {
        spans.push(Span::styled("│ ", theme::dim()));
        spans.push(Span::raw(format!("RSSI {} dBm ", rssi)));
    }
    if let Some(batt) = app.ui.battery_percent {
        spans.push(Span::styled("│ ", theme::dim()));
        spans.push(Span::raw(format!("Bat {}% ", batt)));
    }

    spans.push(Span::styled("│ ", theme::dim()));
    spans.push(Span::raw(format!(
        "{} ",
        Local::now().format("%H:%M:%S")
    )));

    spans.push(Span::styled("│ ", theme::dim()));
    spans.push(Span::styled("?", theme::title()));
    spans.push(Span::raw(" aide "));
    spans.push(Span::styled("q", theme::title()));
    spans.push(Span::raw(" quitter"));

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::dim());
    frame.render_widget(
        Paragraph::new(line)
            .block(block)
            .alignment(Alignment::Left)
            .style(Style::default()),
        area,
    );
}

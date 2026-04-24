use crate::app::App;
use crate::theme;
use crate::util::i18n::{current_lang, t};
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .title(Span::styled(
            format!(" {} ", t("settings.title")),
            theme::title(),
        ));

    let data_dir = app.service.data_dir.to_string_lossy().to_string();

    let body: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ▶ ", theme::title()),
            Span::styled(t("settings.language"), theme::title()),
            Span::raw("   "),
            Span::styled(
                format!("[ {} ]", current_lang().label()),
                theme::ok_style(),
            ),
            Span::styled(
                format!("   {}", t("settings.tab_to_change")),
                theme::dim(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", t("settings.info")),
            theme::title(),
        )),
        Line::from(vec![
            Span::styled(format!("    {}  ", t("settings.version")), theme::dim()),
            Span::raw(env!("CARGO_PKG_VERSION")),
        ]),
        Line::from(vec![
            Span::styled(format!("    {}  ", t("settings.db_path")), theme::dim()),
            Span::raw(data_dir),
        ]),
        Line::from(vec![
            Span::styled(format!("    {}  ", t("settings.state")), theme::dim()),
            Span::styled(
                if app.ui.connected {
                    t("status.connected")
                } else {
                    t("status.disconnected")
                },
                if app.ui.connected {
                    theme::ok_style()
                } else {
                    theme::err_style()
                },
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", t("settings.persist_info1")),
            theme::dim(),
        )),
        Line::from(Span::styled(
            format!("  {}", t("settings.persist_info2")),
            theme::dim(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", t("settings.tab_hint")),
            theme::dim(),
        )),
    ];

    frame.render_widget(
        Paragraph::new(body).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

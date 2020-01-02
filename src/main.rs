use std::io;

use anyhow::Result;
use structopt::StructOpt;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::events::Event;
use crate::widgets::{InfrastrukturSelectionWidget, TermWidget};

mod api;
mod events;
mod widgets;

#[derive(StructOpt, Debug)]
#[structopt(name = "Trassenfinder Infrastructure Explorer")]
struct Opt {
    #[structopt(
        short,
        long,
        default_value = "https://www.trassenfinder.de/api/web/infrastrukturen"
    )]
    api_url: String,
}

fn main() -> Result<()> {
    // Arguments
    let opt: Opt = Opt::from_args();
    let api_url = &opt.api_url;

    // Widgets
    let mut termwidget =
        TermWidget::InfrastrukturSelection(InfrastrukturSelectionWidget::from_url(api_url)?);

    // Terminal
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    let input_events = events::Events::new();

    loop {
        terminal.draw(|mut f| {
            let area = f.size();
            termwidget.render(&mut f, area);
        })?;

        let next_termwidget = match input_events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    terminal.clear()?;
                    break;
                }
                key @ _ => match termwidget {
                    TermWidget::InfrastrukturSelection(ref mut widget) => {
                        widget.select_key(key, api_url)
                    }
                    TermWidget::Map(ref mut widget) => widget.select_key(key, api_url),
                },
            },
        }?;

        if let Some(next_widget) = next_termwidget {
            termwidget = next_widget;
        }
    }

    Ok(())
}

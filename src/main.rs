use std::io;

use anyhow::Result;
use structopt::StructOpt;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::widgets::Widget;
use tui::Terminal;

use crate::events::Event;
use crate::widgets::{InfrastrukturSelectionWidget, MapWidget};

mod api;
mod events;
mod widgets;

enum TermWidget {
    InfrastrukturSelection(InfrastrukturSelectionWidget),
    Map(MapWidget),
}

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

#[tokio::main]
async fn main() -> Result<()> {
    let opt: Opt = Opt::from_args();

    // Widgets
    let mut termwidget = TermWidget::InfrastrukturSelection(
        InfrastrukturSelectionWidget::from_url(&opt.api_url).await?,
    );

    // Terminal
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    let input_events = events::Events::new();

    loop {
        terminal.draw(|mut f| {
            let rect = f.size();

            match termwidget {
                TermWidget::InfrastrukturSelection(ref mut widget) => widget.render(&mut f, rect),
                TermWidget::Map(ref mut widget) => widget.render(&mut f, rect),
            }
        })?;

        match input_events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    terminal.clear()?;
                    break;
                }
                key @ _ => match termwidget {
                    TermWidget::InfrastrukturSelection(ref mut widget) => {
                        if let Some(infrastruktur_index) = widget.key_select(key) {
                            let map_widget =
                                MapWidget::from_url(&opt.api_url, infrastruktur_index.id).await?;
                            termwidget = TermWidget::Map(map_widget);
                        }
                    }
                    TermWidget::Map(ref mut widget) => widget.key_select(key),
                },
            },
        }
    }

    Ok(())
}

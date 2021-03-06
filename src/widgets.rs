use anyhow::Result;
use ordered_float::OrderedFloat;
use termion::event::Key;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line, Points};
use tui::widgets::{Block, Borders, SelectableList, Widget};

use crate::api::{
    read_infrastructure_infos, read_station_map, InfrastrukturInfo, Segment, Station, StationMap,
};
use tui::backend::Backend;
use tui::Frame;

pub enum TermWidget {
    InfrastrukturSelection(InfrastrukturSelectionWidget),
    Map(MapWidget),
}

impl TermWidget {
    pub fn render<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        Self: Sized,
        B: Backend,
    {
        match *self {
            TermWidget::InfrastrukturSelection(ref mut widget) => widget.render(f, area),
            TermWidget::Map(ref mut widget) => widget.render(f, area),
        }
    }
}

//
// InfrastrukturSelectionWidget
//

pub struct InfrastrukturSelectionWidget {
    values: Vec<InfrastrukturInfo>,
    items: Vec<String>,
    selected: Option<usize>,
}

impl InfrastrukturSelectionWidget {
    pub fn new(values: Vec<InfrastrukturInfo>) -> Self {
        let items = values
            .iter()
            .map(|index| format!("{}: {}", index.id, index.anzeigename))
            .collect::<Vec<_>>();

        let selected = if values.is_empty() {
            None
        } else {
            Some(0usize)
        };

        InfrastrukturSelectionWidget {
            values,
            items,
            selected,
        }
    }

    pub fn from_url(url: &str) -> Result<Self> {
        Ok(Self::new(read_infrastructure_infos(url)?))
    }

    fn up(&mut self) {
        self.selected = up(&self.values, self.selected);
    }

    fn down(&mut self) {
        self.selected = down(&self.values, self.selected);
    }

    fn selected_value(&self) -> Option<&InfrastrukturInfo> {
        if let Some(index) = self.selected {
            Some(&self.values[index])
        } else {
            None
        }
    }

    pub fn select_key(&mut self, key: Key, api_url: &str) -> Result<Option<TermWidget>> {
        match key {
            Key::Up => self.up(),
            Key::Down => self.down(),
            Key::Char('\n') /* enter */ => {
                if let Some(info) = self.selected_value() {
                    return Ok(Some(TermWidget::Map(MapWidget::from_url(api_url, info.id)?)));
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Widget for InfrastrukturSelectionWidget {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(area)[1];

        let rect = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(rect)[1];

        SelectableList::default()
            .block(
                Block::default()
                    .title("Infrastrukturen")
                    .borders(Borders::ALL),
            )
            .items(&self.items[..])
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().modifier(Modifier::BOLD))
            .select(self.selected)
            .draw(rect, buf);
    }
}

//
// MapWidget
//

#[derive(PartialEq)]
enum WidgetSelection {
    Stations,
    Segments,
}

pub struct MapWidget {
    station_map: StationMap,
    coordinates: Vec<(f64, f64)>,
    extent: Extent,

    stations_widget: ListSelectionWidget,
    segments_widget: ListSelectionWidget,

    widget_selection: WidgetSelection,
}

impl MapWidget {
    pub fn new(station_map: StationMap) -> Self {
        let station_names: Vec<String> = station_map
            .stations
            .iter()
            .map(|station| format!("{} ({})", station.ds100, station.longname))
            .collect();

        let stations_widget: ListSelectionWidget =
            ListSelectionWidget::new("Betriebsstellen".to_string(), station_names);

        let segment_names: Vec<String> = station_map
            .segments
            .iter()
            .map(|segment| {
                format!(
                    "{} ({} -> {})",
                    segment.routenumber, segment.from.ds100, segment.to.ds100
                )
            })
            .collect();

        let segments_widget: ListSelectionWidget =
            ListSelectionWidget::new("Streckensegmente".to_string(), segment_names);

        let coordinates = station_map.coordinates();
        let extent = calc_extent(&coordinates);

        MapWidget {
            station_map,
            coordinates,
            extent,
            stations_widget,
            segments_widget,
            widget_selection: WidgetSelection::Stations,
        }
    }

    pub fn from_url(bae_url: &str, id: u64) -> Result<Self> {
        Ok(Self::new(read_station_map(bae_url, id)?))
    }

    pub fn select_key(&mut self, key: Key, api_url: &str) -> Result<Option<TermWidget>> {
        match key {
            Key::Char('b') => self.widget_selection = WidgetSelection::Stations,
            Key::Char('s') => self.widget_selection = WidgetSelection::Segments,
            Key::Esc => {
                let widget = InfrastrukturSelectionWidget::from_url(api_url)?;
                return Ok(Some(TermWidget::InfrastrukturSelection(widget)));
            }
            _ => {}
        }

        match self.widget_selection {
            WidgetSelection::Stations => self.stations_widget.select_key(key),
            WidgetSelection::Segments => self.segments_widget.select_key(key),
        }
    }
}

impl Widget for MapWidget {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        let left_rect = h_chunks[0];
        let right_rect = h_chunks[1];

        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(left_rect);

        let top_left = v_chunks[0];
        let bottom_left = v_chunks[1];

        self.stations_widget.draw(top_left, buf);
        self.segments_widget.draw(bottom_left, buf);

        let selected_station: Option<&Station> = self
            .stations_widget
            .selected
            .and_then(|index| self.station_map.stations.get(index));
        let selected_segment: Option<&Segment> = self
            .segments_widget
            .selected
            .and_then(|index| self.station_map.segments.get(index));

        Canvas::default()
            .block(Block::default().title("Karte").borders(Borders::ALL))
            .x_bounds([self.extent.min_x, self.extent.max_x])
            .y_bounds([self.extent.min_y, self.extent.max_y])
            .paint(|ctx| {
                ctx.draw(&Points {
                    coords: &self.coordinates[..],
                    color: Color::Blue,
                });

                if let Some(station) = selected_station {
                    ctx.layer();
                    ctx.draw(&Points {
                        coords: &[(station.coord.0, station.coord.1)],
                        color: Color::Red,
                    });
                }

                if let Some(segment) = selected_segment {
                    ctx.layer();
                    ctx.draw(&Line {
                        x1: segment.from.coord.0,
                        y1: segment.from.coord.1,
                        x2: segment.to.coord.0,
                        y2: segment.to.coord.1,
                        color: Color::Yellow,
                    });
                }
            })
            .draw(right_rect, buf);
    }
}

#[derive(Debug)]
struct Extent {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

fn calc_extent(coords: &[(f64, f64)]) -> Extent {
    // x coordinates
    let x_coords: Vec<OrderedFloat<f64>> =
        coords.iter().map(|coord| OrderedFloat(coord.0)).collect();
    let y_coords: Vec<OrderedFloat<f64>> =
        coords.iter().map(|coord| OrderedFloat(coord.1)).collect();

    Extent {
        min_x: x_coords.iter().min().unwrap().clone().into(),
        max_x: x_coords.iter().max().unwrap().clone().into(),
        min_y: y_coords.iter().min().unwrap().clone().into(),
        max_y: y_coords.iter().max().unwrap().clone().into(),
    }
}

//
// ListSelectionWidget
//

struct ListSelectionWidget {
    title: String,
    names: Vec<String>,
    selected: Option<usize>,
}

impl Widget for ListSelectionWidget {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        SelectableList::default()
            .block(Block::default().title(&self.title).borders(Borders::ALL))
            .items(&self.names[..])
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().modifier(Modifier::BOLD))
            .select(self.selected)
            .draw(area, buf);
    }
}

impl ListSelectionWidget {
    pub fn new(title: String, names: Vec<String>) -> Self {
        let selected = if names.is_empty() { None } else { Some(0usize) };

        ListSelectionWidget {
            title,
            names,
            selected,
        }
    }

    fn up(&mut self) {
        self.selected = up(&self.names, self.selected);
    }

    fn down(&mut self) {
        self.selected = down(&self.names, self.selected);
    }

    pub fn select_key(&mut self, key: Key) -> Result<Option<TermWidget>> {
        match key {
            Key::Up => self.up(),
            Key::Down => self.down(),
            _ => {}
        }
        Ok(None)
    }
}

fn up<T>(values: &[T], selected: Option<usize>) -> Option<usize> {
    if values.is_empty() {
        return None;
    }

    if let Some(index) = selected {
        if index > 0 {
            Some(index - 1)
        } else {
            Some(index)
        }
    } else {
        Some(0)
    }
}

fn down<T>(values: &[T], selected: Option<usize>) -> Option<usize> {
    if values.is_empty() {
        return None;
    }

    let max_index = values.len() - 1;
    if let Some(index) = selected {
        if index < max_index {
            Some(index + 1)
        } else {
            Some(index)
        }
    } else {
        Some(0)
    }
}

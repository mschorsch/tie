use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

//
// StationMap
//

#[derive(Debug)]
pub struct StationMap {
    pub id: u64,
    pub name: String,
    pub stations: Vec<Station>,
    pub segments: Vec<Segment>,
}

impl StationMap {
    pub fn coordinates(&self) -> Vec<(f64, f64)> {
        self.stations.iter().map(|station| station.coord).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Station {
    pub ds100: String,
    pub longname: String,
    pub coord: (f64, f64), // (x, y)
}

#[derive(Debug)]
pub struct Segment {
    pub from: Station,
    pub to: Station,
    pub routenumber: u32,
}

//
// API
//

#[derive(Deserialize, Debug)]
pub struct InfrastrukturIndex {
    pub id: u64,
    pub anzeigename: String,
    pub fahrplanjahr: u32,
    pub gueltig_von: String,
    pub gueltig_bis: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Infrastruktur {
    pub id: u64,
    pub anzeigename: String,
    pub ordnungsrahmen: Ordnungsrahmen,
    // ...
}

impl TryInto<StationMap> for Infrastruktur {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StationMap> {
        let betriebsstellen = self.ordnungsrahmen.betriebsstellen;
        let streckensegmente = self.ordnungsrahmen.streckensegmente;

        // Stations
        let stations: Vec<Station> = betriebsstellen
            .into_iter()
            .map(|bst| Station {
                ds100: bst.ds100,
                longname: bst.langname,
                coord: (bst.x, bst.y),
            })
            .collect();

        // Stations-Index-Map
        let mut stations_index_map: HashMap<&str, Station> = HashMap::new();
        for station in &stations {
            stations_index_map.insert(station.ds100.as_str(), station.clone());
        }

        // Segments
        let mut segments = Vec::with_capacity(streckensegmente.len());
        for streckensegment in streckensegmente {
            let from = (*stations_index_map
                .get(&streckensegment.von.as_ref())
                .with_context(|| {
                    format!(
                        "Station '{}' for Segment '{}' not found",
                        streckensegment.von,
                        streckensegment.to_string()
                    )
                })?)
            .clone();
            let to = (*stations_index_map
                .get(&streckensegment.bis.as_str())
                .with_context(|| {
                    format!(
                        "Station '{}' for Segment '{}' not found",
                        streckensegment.bis,
                        streckensegment.to_string()
                    )
                })?)
            .clone();
            let routenumber = streckensegment.streckennummer;
            segments.push(Segment {
                from,
                to,
                routenumber,
            });
        }

        Ok(StationMap {
            id: self.id,
            name: self.anzeigename,
            stations,
            segments,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ordnungsrahmen {
    pub betriebsstellen: Vec<Betriebsstelle>,
    // mutter_betriebsstellen
    pub streckensegmente: Vec<Streckensegment>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Betriebsstelle {
    pub x: f64,
    pub y: f64,
    pub ds100: String,

    #[serde(rename = "langname_stammdaten")]
    pub langname: String,
    // ...
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Streckensegment {
    pub von: String,
    pub bis: String,
    pub streckennummer: u32,
    // ...
}

impl Streckensegment {
    pub fn to_string(&self) -> String {
        format!("{}-{}-{}", self.von, self.streckennummer, self.bis)
    }
}

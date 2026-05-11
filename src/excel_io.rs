use std::collections::HashMap;

use rand::prelude::*;
use umya_spreadsheet::{self, reader, writer};

use crate::generator::{SeatingChart, Zone};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellPos {
    pub row: u32,
    pub col: u32,
}

pub struct ZoneCellConfig {
    pub zone1: Vec<CellPos>,
    pub zone2a: Vec<CellPos>,
    pub zone2b: Vec<CellPos>,
    pub zone3: Vec<CellPos>,
}

impl ZoneCellConfig {
    pub fn to_capacities(&self) -> HashMap<Zone, usize> {
        HashMap::from([
            (Zone::Zone1, self.zone1.len()),
            (Zone::Zone2a, self.zone2a.len()),
            (Zone::Zone2b, self.zone2b.len()),
            (Zone::Zone3, self.zone3.len()),
        ])
    }

    pub fn from_template(path: &str) -> Result<Self, String> {
        let book = reader::xlsx::read(path)
            .map_err(|e| format!("failed to read template '{}': {}", path, e))?;
        let sheet = book
            .get_sheet(&0)
            .ok_or_else(|| format!("no sheet in '{}'", path))?;

        let max_row = sheet.get_highest_row();
        let max_col = sheet.get_highest_column();

        let mut zone1 = Vec::new();
        let mut zone2a = Vec::new();
        let mut zone2b = Vec::new();
        let mut zone3 = Vec::new();

        for row in 1..=max_row {
            for col in 1..=max_col {
                let value = sheet.get_value((col, row));
                match value.trim() {
                    "1" => zone1.push(CellPos { row, col }),
                    "2a" => zone2a.push(CellPos { row, col }),
                    "2b" => zone2b.push(CellPos { row, col }),
                    "3" => zone3.push(CellPos { row, col }),
                    _ => {}
                }
            }
        }

        let total = zone1.len() + zone2a.len() + zone2b.len() + zone3.len();
        if total == 0 {
            return Err("no zone markers (1, 2a, 2b, 3) found in template".to_string());
        }

        Ok(ZoneCellConfig {
            zone1,
            zone2a,
            zone2b,
            zone3,
        })
    }

    fn all_zones(&self) -> [(Zone, &[CellPos]); 4] {
        [
            (Zone::Zone1, self.zone1.as_slice()),
            (Zone::Zone2a, self.zone2a.as_slice()),
            (Zone::Zone2b, self.zone2b.as_slice()),
            (Zone::Zone3, self.zone3.as_slice()),
        ]
    }
}

pub fn read_chart(path: &str, config: &ZoneCellConfig) -> Result<SeatingChart, String> {
    let book =
        reader::xlsx::read(path).map_err(|e| format!("failed to read '{}': {}", path, e))?;

    let sheet = book
        .get_sheet(&0)
        .ok_or_else(|| format!("no sheet found in '{}'", path))?;

    let mut assignments = HashMap::new();

    for (zone, cells) in config.all_zones().iter() {
        for pos in *cells {
            let value = sheet.get_value((pos.col, pos.row));
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                assignments.insert(trimmed.to_string(), *zone);
            }
        }
    }

    Ok(SeatingChart::new(assignments))
}

pub fn write_chart(
    template_path: &str,
    output_path: &str,
    chart: &SeatingChart,
    config: &ZoneCellConfig,
) -> Result<(), String> {
    let mut rng = rand::rng();
    let mut book = reader::xlsx::read(template_path)
        .map_err(|e| format!("failed to read template '{}': {}", template_path, e))?;

    let sheet = book
        .get_sheet_mut(&0)
        .ok_or_else(|| "no sheet found in template".to_string())?;

    for (zone, cells) in config.all_zones().iter() {
        let mut people: Vec<&str> = chart
            .assignments
            .iter()
            .filter(|(_, z)| *z == zone)
            .map(|(p, _)| p.as_str())
            .collect();
        people.shuffle(&mut rng);

        for (i, pos) in cells.iter().enumerate() {
            if i >= people.len() {
                break;
            }
            let cell = sheet.get_cell_mut((pos.col, pos.row));
            cell.set_value(people[i]);
        }
    }

    writer::xlsx::write(&book, output_path)
        .map_err(|e| format!("failed to write '{}': {}", output_path, e))?;

    Ok(())
}

use std::collections::HashMap;

use rand::seq::SliceRandom;
use umya_spreadsheet::{self, reader, writer};

use crate::generator::{SeatingChart, Zone};

pub struct ZoneCellConfig<'a> {
    pub zone1: Vec<&'a str>,
    pub zone2a: Vec<&'a str>,
    pub zone2b: Vec<&'a str>,
    pub zone3: Vec<&'a str>,
}

impl ZoneCellConfig<'_> {
    pub fn to_capacities(&self) -> HashMap<Zone, usize> {
        HashMap::from([
            (Zone::Zone1, self.zone1.len()),
            (Zone::Zone2a, self.zone2a.len()),
            (Zone::Zone2b, self.zone2b.len()),
            (Zone::Zone3, self.zone3.len()),
        ])
    }

    fn all_zones(&self) -> [(Zone, &[&str]); 4] {
        [
            (Zone::Zone1, self.zone1.as_slice()),
            (Zone::Zone2a, self.zone2a.as_slice()),
            (Zone::Zone2b, self.zone2b.as_slice()),
            (Zone::Zone3, self.zone3.as_slice()),
        ]
    }
}

pub fn read_chart(path: &str, config: &ZoneCellConfig) -> Result<SeatingChart, String> {
    let book = reader::xlsx::read(path).map_err(|e| format!("failed to read '{}': {}", path, e))?;

    let sheet = book
        .get_sheet(&0)
        .ok_or_else(|| format!("no sheet found in '{}'", path))?;

    let mut assignments = HashMap::new();

    for (zone, cells) in config.all_zones().iter() {
        for cell_ref in *cells {
            let value = sheet.get_value(*cell_ref);
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
        people.shuffle(&mut rand::rng());

        for (i, cell_ref) in cells.iter().enumerate() {
            if i >= people.len() {
                break;
            }
            let cell = sheet.get_cell_mut(*cell_ref);
            cell.set_value(people[i]);
        }
    }

    writer::xlsx::write(&book, output_path)
        .map_err(|e| format!("failed to write '{}': {}", output_path, e))?;

    Ok(())
}

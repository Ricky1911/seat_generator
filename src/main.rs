mod excel_io;
mod generator;

use excel_io::{ZoneCellConfig, read_chart, write_chart};
use generator::Generator;

fn main() {
    let config = ZoneCellConfig {
        zone1: vec![
            "D1", "E1", "G1", "H1", "A2", "B2", "D2", "E2", "G2", "H2", "J2", "K2",
        ],
        zone2a: vec!["A3", "B3", "A4", "B4", "A5", "B5", "A6", "B6"],
        zone2b: vec!["J3", "K3", "J4", "K4", "J5", "K5", "J6", "K6"],
        zone3: vec![
            "D3", "E3", "G3", "H3", "D4", "E4", "G4", "H4", "D5", "E5", "G5", "H5", "D6", "E6",
            "G6", "H6",
        ],
    };

    let generator = Generator::new(config.to_capacities());

    let chart1 = read_chart("history1.xlsx", &config).expect("failed to read history1.xlsx");
    let chart2 = read_chart("history2.xlsx", &config).expect("failed to read history2.xlsx");

    match generator.generate(&chart1, &chart2) {
        Ok(chart) => {
            write_chart("template.xlsx", "output.xlsx", &chart, &config)
                .expect("failed to write output.xlsx");
            println!("done -> output.xlsx");
        }
        Err(e) => eprintln!("error: {}", e),
    }
}

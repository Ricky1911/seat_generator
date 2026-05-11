mod excel_io;
mod generator;

use excel_io::{read_chart, write_chart, ZoneCellConfig};
use generator::Generator;

fn main() {
    let config = ZoneCellConfig::from_template("template.xlsx")
        .expect("failed to read template.xlsx");

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

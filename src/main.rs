#![feature(path_is_empty)]
use std::path::Path;

mod config;
mod excel_io;
mod generator;
mod gui;
mod server;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--cli" {
        run_cli();
    } else {
        run_gui();
    }
}

fn run_cli() {
    use excel_io::{ZoneCellConfig, read_chart, write_chart};
    use generator::Generator;

    if server::try_fetch_from_server(
        Path::new("template.xlsx"),
        Path::new("history1.xlsx"),
        Path::new("history2.xlsx"),
        Path::new("output.xlsx"),
    ) {
        println!("done -> output.xlsx");
        return;
    }

    let config = ZoneCellConfig::from_template(Path::new("template.xlsx"))
        .expect("failed to read template.xlsx");

    let generator = Generator::new(config.to_capacities());

    let chart1 =
        read_chart(Path::new("history1.xlsx"), &config).expect("failed to read history1.xlsx");
    let chart2 =
        read_chart(Path::new("history2.xlsx"), &config).expect("failed to read history2.xlsx");

    match generator.generate(&chart1, &chart2) {
        Ok(chart) => {
            write_chart(
                Path::new("template.xlsx"),
                Path::new("output.xlsx"),
                &chart,
                &config,
            )
            .expect("failed to write output.xlsx");
            println!("done -> output.xlsx");
        }
        Err(e) => eprintln!("error: {}", e),
    }
}

fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 480.0])
            .with_min_inner_size([400.0, 280.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Seat Generator",
        options,
        Box::new(|_cc| Ok(Box::new(gui::SeatGeneratorApp::new()))),
    )
    .expect("failed to start GUI");
}

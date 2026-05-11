use eframe::egui;
use crate::excel_io::{read_chart, write_chart, ZoneCellConfig};
use crate::generator::Generator;

pub struct SeatGeneratorApp {
    template_path: String,
    history1_path: String,
    history2_path: String,
    output_path: String,
    status: String,
    status_is_error: bool,
}

impl SeatGeneratorApp {
    pub fn new() -> Self {
        let config = crate::config::Config::load();
        Self {
            template_path: config.template_path,
            history1_path: config.history1_path,
            history2_path: config.history2_path,
            output_path: config.output_path,
            status: "Ready".to_string(),
            status_is_error: false,
        }
    }

    fn save_config(&self) {
        crate::config::Config {
            template_path: self.template_path.clone(),
            history1_path: self.history1_path.clone(),
            history2_path: self.history2_path.clone(),
            output_path: self.output_path.clone(),
        }
        .save();
    }

    fn pick_excel_file(current: &str) -> Option<String> {
        let mut dialog = rfd::FileDialog::new().add_filter("Excel", &["xlsx"]);
        if let Some(parent) = std::path::Path::new(current).parent() {
            if parent.exists() {
                dialog = dialog.set_directory(parent);
            }
        }
        dialog.pick_file().map(|p| p.to_string_lossy().to_string())
    }

    fn pick_save_file(current: &str) -> Option<String> {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Excel", &["xlsx"])
            .set_file_name("output.xlsx");
        if let Some(parent) = std::path::Path::new(current).parent() {
            if parent.exists() {
                dialog = dialog.set_directory(parent);
            }
        }
        dialog.save_file().map(|p| p.to_string_lossy().to_string())
    }

    fn run_generation(&mut self) {
        if self.template_path.is_empty() || self.history1_path.is_empty()
            || self.history2_path.is_empty() || self.output_path.is_empty()
        {
            self.status = "Error: all four file paths must be specified".to_string();
            self.status_is_error = true;
            return;
        }

        let config = match ZoneCellConfig::from_template(&self.template_path) {
            Ok(c) => c,
            Err(e) => {
                self.status = format!("Failed to read template: {}", e);
                self.status_is_error = true;
                return;
            }
        };

        let generator = Generator::new(config.to_capacities());

        let chart1 = match read_chart(&self.history1_path, &config) {
            Ok(c) => c,
            Err(e) => {
                self.status = format!("Failed to read history 1: {}", e);
                self.status_is_error = true;
                return;
            }
        };

        let chart2 = match read_chart(&self.history2_path, &config) {
            Ok(c) => c,
            Err(e) => {
                self.status = format!("Failed to read history 2: {}", e);
                self.status_is_error = true;
                return;
            }
        };

        match generator.generate(&chart1, &chart2) {
            Ok(chart) => {
                match write_chart(&self.template_path, &self.output_path, &chart, &config) {
                    Ok(()) => {
                        self.status = format!("Done -> {}", self.output_path);
                        self.status_is_error = false;
                        self.save_config();
                    }
                    Err(e) => {
                        self.status = format!("Failed to write output: {}", e);
                        self.status_is_error = true;
                    }
                }
            }
            Err(e) => {
                self.status = format!("Generation failed: {}", e);
                self.status_is_error = true;
            }
        }
    }
}

impl eframe::App for SeatGeneratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("info_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("File Info");
                ui.separator();
                ui.add_space(6.0);

                ui.label(egui::RichText::new("Template").strong());
                ui.label("  Template file. Cells with values");
                ui.label("  1 / 2a / 2b / 3 mark zones.");
                ui.label("  Each zone defines assignable");
                ui.label("  seats and total capacity.");
                ui.add_space(14.0);

                ui.label(egui::RichText::new("History 1").strong());
                ui.label("  Most recent seating chart.");
                ui.label("  A person cannot sit in the");
                ui.label("  same zone twice in a row");
                ui.label("  (Zone1/2a/2b restricted).");
                ui.add_space(14.0);

                ui.label(egui::RichText::new("History 2").strong());
                ui.label("  Second most recent chart.");
                ui.label("  Combined with History 1 to");
                ui.label("  enforce the Zone 2 limit:");
                ui.label("  max 2 consecutive sits in");
                ui.label("  Zone 2a + 2b combined.");
                ui.add_space(14.0);

                ui.label(egui::RichText::new("Output").strong());
                ui.label("  Where to save the generated");
                ui.label("  seating chart (.xlsx).");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Seat Generator");
            ui.separator();

            ui.vertical_centered(|ui| {
                ui.set_max_width(640.0);
                egui::Grid::new("file_grid")
                    .num_columns(3)
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.label("Template:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.template_path)
                                .desired_width(f32::INFINITY),
                        );
                        if ui.button("Browse...").clicked() {
                            ctx.request_repaint();
                            if let Some(path) = Self::pick_excel_file(&self.template_path) {
                                self.template_path = path;
                            }
                        }
                        ui.end_row();

                        ui.label("History 1:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.history1_path)
                                .desired_width(f32::INFINITY),
                        );
                        if ui.button("Browse...").clicked() {
                            ctx.request_repaint();
                            if let Some(path) = Self::pick_excel_file(&self.history1_path) {
                                self.history1_path = path;
                            }
                        }
                        ui.end_row();

                        ui.label("History 2:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.history2_path)
                                .desired_width(f32::INFINITY),
                        );
                        if ui.button("Browse...").clicked() {
                            ctx.request_repaint();
                            if let Some(path) = Self::pick_excel_file(&self.history2_path) {
                                self.history2_path = path;
                            }
                        }
                        ui.end_row();

                        ui.label("Output:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.output_path)
                                .desired_width(f32::INFINITY),
                        );
                        if ui.button("Save As...").clicked() {
                            ctx.request_repaint();
                            if let Some(path) = Self::pick_save_file(&self.output_path) {
                                self.output_path = path;
                            }
                        }
                        ui.end_row();
                    });
            });

            ui.add_space(12.0);

            ui.vertical_centered(|ui| {
                let button = egui::Button::new("Generate")
                    .min_size(egui::vec2(160.0, 36.0));
                if ui.add(button).clicked() {
                    self.run_generation();
                }
            });

            ui.add_space(8.0);
            ui.separator();

            if self.status_is_error {
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), &self.status);
            } else {
                ui.label(&self.status);
            }
        });
    }
}

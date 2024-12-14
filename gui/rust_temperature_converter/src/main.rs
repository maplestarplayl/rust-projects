use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Temperature Converter",
        options,
        Box::new(|_cc| Ok((Box::new(TempConverter::default())))),
    )
}

struct TempConverter {
    celsius: f64,
    fahrenheit: f64,
}

impl Default for TempConverter {
    fn default() -> Self {
        Self { celsius: 0.0, fahrenheit: 32.0 }
    }
}

impl eframe::App for TempConverter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.heading("Temperature Converter");
            ui.separator();

            ui.label("Celsius");
            ui.text_edit_singleline(&mut self.celsius.to_string());

            if ui.button("Convert to Fahrenheit").clicked() {
                self.fahrenheit = self.celsius * 1.8 + 32.0;
            }
            
            ui.label("Fahrenheit");
            ui.text_edit_singleline(&mut self.fahrenheit.to_string());

            if ui.button("Convert to Celsius").clicked() {
                self.celsius = (self.fahrenheit - 32.0) / 1.8;
            }
        });
    }
}
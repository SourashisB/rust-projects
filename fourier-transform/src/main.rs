use eframe::egui;
use egui::{plot::{Line, Plot, PlotPoints}, CentralPanel, Context};
use rustfft::{FftPlanner, num_complex::Complex};
use meval::Expr;

pub struct FourierApp {
    input_function: String,
    samples: usize,
    spectrum: Option<Vec<(f64, f64)>>,
    error: Option<String>,
}

impl Default for FourierApp {
    fn default() -> Self {
        Self {
            input_function: "sin(2 * pi * x)".to_string(),
            samples: 512,
            spectrum: None,
            error: None,
        }
    }
}

impl eframe::App for FourierApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Fourier Transform Visualizer");

            ui.horizontal(|ui| {
                ui.label("f(x) =");
                ui.text_edit_singleline(&mut self.input_function);
            });

            ui.add(egui::Slider::new(&mut self.samples, 64..=2048).text("Samples"));

            if ui.button("Compute Fourier Transform").clicked() {
                self.compute_fft();
            }

            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Some(ref spectrum) = self.spectrum {
                let points: PlotPoints = spectrum
                    .iter()
                    .map(|(i, mag)| [*i, *mag])
                    .collect();

                Plot::new("fft_plot")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new(points).name("Magnitude Spectrum"));
                    });
            }
        });
    }
}

impl FourierApp {
    fn compute_fft(&mut self) {
        self.error = None;
        self.spectrum = None;

        let expr = match self.input_function.parse::<Expr>() {
            Ok(e) => e,
            Err(e) => {
                self.error = Some(format!("Parse error: {}", e));
                return;
            }
        };

        let func = match expr.bind("x") {
            Ok(f) => f,
            Err(e) => {
                self.error = Some(format!("Function bind error: {}", e));
                return;
            }
        };

        let n = self.samples;
        let dt = 1.0 / n as f64;

        let mut input: Vec<Complex<f64>> = (0..n)
            .map(|i| {
                let x = i as f64 * dt;
                Complex::new(func(x), 0.0)
            })
            .collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        fft.process(&mut input);

        let spectrum: Vec<(f64, f64)> = input
            .iter()
            .take(n / 2)
            .enumerate()
            .map(|(i, c)| (i as f64, c.norm()))
            .collect();

        self.spectrum = Some(spectrum);
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Fourier Transform Visualizer",
        options,
        Box::new(|_cc| Box::<FourierApp>::default()),
    )
}
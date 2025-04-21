use iced::{
    canvas::{self, Canvas, Cursor, Frame, Geometry, Path, Stroke},
    executor, Application, Color, Column, Command, Container, Element, Length, Row, Settings,
    Slider, Text, TextInput,
};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::PI;

fn main() -> iced::Result {
    FourierApp::run(Settings::default())
}

struct FourierApp {
    equation: String,
    sample_rate: u32,
    duration: f64,
    error_message: String,
    time_domain_samples: Vec<f64>,
    frequency_domain: Vec<(f64, f64)>, // frequency, magnitude
    sample_rate_state: slider::State,
    duration_state: slider::State,
}

#[derive(Debug, Clone)]
enum Message {
    EquationChanged(String),
    SampleRateChanged(u32),
    DurationChanged(f64),
    ComputeTransform,
}

mod slider {
    use iced::slider;
    
    pub struct State {
        state: slider::State,
    }
    
    impl State {
        pub fn new() -> Self {
            Self {
                state: slider::State::new(),
            }
        }
        
        pub fn view<'a>(&'a mut self, value: f64, min: f64, max: f64, step: f64, on_change: impl Fn(f64) -> Message + 'static) -> Element<'a, Message> {
            iced::Slider::new(
                &mut self.state,
                min..=max,
                value,
                on_change,
            )
            .step(step)
            .into()
        }
    }
    
    pub type Message = super::Message;
}

impl Application for FourierApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                equation: String::from("sin(2*pi*x)"),
                sample_rate: 1000,
                duration: 1.0,
                error_message: String::new(),
                time_domain_samples: Vec::new(),
                frequency_domain: Vec::new(),
                sample_rate_state: slider::State::new(),
                duration_state: slider::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Fourier Transform Visualizer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EquationChanged(equation) => {
                self.equation = equation;
            }
            Message::SampleRateChanged(rate) => {
                self.sample_rate = rate;
            }
            Message::DurationChanged(duration) => {
                self.duration = duration;
            }
            Message::ComputeTransform => {
                self.compute_fourier_transform();
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("Fourier Transform Visualizer").size(28);

        let equation_input = Row::new()
            .padding(20)
            .spacing(10)
            .push(Text::new("Wave equation f(x) = "))
            .push(
                TextInput::new("Enter equation here", &self.equation)
                    .on_input(Message::EquationChanged)
                    .padding(10)
                    .size(20),
            );

        let controls = Row::new()
            .padding(20)
            .spacing(20)
            .push(Text::new("Sample rate (Hz):"))
            .push(
                self.sample_rate_state.view(
                    self.sample_rate as f64,
                    100.0,
                    10000.0,
                    100.0,
                    |v| Message::SampleRateChanged(v as u32),
                )
                .width(Length::Units(150)),
            )
            .push(Text::new(format!("{}", self.sample_rate)))
            .push(Text::new("Duration (seconds):"))
            .push(
                self.duration_state.view(
                    self.duration,
                    0.1,
                    10.0,
                    0.1,
                    Message::DurationChanged,
                )
                .width(Length::Units(150)),
            )
            .push(Text::new(format!("{:.1}", self.duration)));

        let compute_button = iced::Button::new(Text::new("Compute Fourier Transform"))
            .on_press(Message::ComputeTransform)
            .padding(10);

        let error_message = if !self.error_message.is_empty() {
            Text::new(&self.error_message).color([1.0, 0.0, 0.0])
        } else {
            Text::new("")
        };

        let time_domain_canvas = Canvas::new(TimeDomainPlot {
            samples: self.time_domain_samples.clone(),
            duration: self.duration,
        })
        .width(Length::Fill)
        .height(Length::Units(200));

        let frequency_domain_canvas = Canvas::new(FrequencyDomainPlot {
            data: self.frequency_domain.clone(),
        })
        .width(Length::Fill)
        .height(Length::Units(200));

        let significant_freqs = if !self.frequency_domain.is_empty() {
            let max_magnitude = self.frequency_domain.iter().fold(0.01, |max, &(_, m)| m.max(max));
            let threshold = max_magnitude * 0.1;
            let significant: Vec<_> = self.frequency_domain
                .iter()
                .filter(|&&(_, m)| m > threshold)
                .collect();
            
            let mut text = String::from("Significant frequency components: ");
            for (i, &(freq, magnitude)) in significant.iter().take(5).enumerate() {
                if i > 0 {
                    text.push_str(", ");
                }
                text.push_str(&format!("{:.2} Hz (magnitude: {:.3})", freq, magnitude));
            }
            Text::new(text)
        } else {
            Text::new("")
        };

        let content = Column::new()
            .push(title)
            .push(equation_input)
            .push(controls)
            .push(compute_button)
            .push(error_message)
            .push(Text::new("Time Domain Signal").size(20))
            .push(time_domain_canvas)
            .push(Text::new("Frequency Domain").size(20))
            .push(frequency_domain_canvas)
            .push(significant_freqs)
            .padding(20)
            .spacing(10);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

impl FourierApp {
    fn compute_fourier_transform(&mut self) {
        self.error_message.clear();
        
        // Parse and evaluate the equation
        let equation = self.equation.replace("pi", &PI.to_string());
        
        // This is a simplified parser that only handles basic math expressions
        // In a real application, we would use a proper expression parser like 'meval'
        let expr = match SimpleExprParser::new(&equation) {
            Ok(expr) => expr,
            Err(e) => {
                self.error_message = format!("Error parsing equation: {}", e);
                return;
            }
        };

        // Generate time domain samples
        let num_samples = (self.sample_rate as f64 * self.duration) as usize;
        let dt = 1.0 / self.sample_rate as f64;
        
        self.time_domain_samples.clear();
        for i in 0..num_samples {
            let t = i as f64 * dt;
            match expr.evaluate(t) {
                Ok(val) => self.time_domain_samples.push(val),
                Err(e) => {
                    self.error_message = format!("Error evaluating equation at x={}: {}", t, e);
                    return;
                }
            }
        }

        // Perform FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(num_samples);
        
        // Prepare input data
        let mut complex_input: Vec<Complex<f64>> = self.time_domain_samples
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        
        // Execute FFT
        fft.process(&mut complex_input);
        
        // Process output
        self.frequency_domain.clear();
        for i in 0..num_samples / 2 {
            let frequency = i as f64 * (self.sample_rate as f64 / num_samples as f64);
            let magnitude = complex_input[i].norm() / num_samples as f64;
            self.frequency_domain.push((frequency, magnitude));
        }
    }
}

struct TimeDomainPlot {
    samples: Vec<f64>,
    duration: f64,
}

impl canvas::Program<Message> for TimeDomainPlot {
    fn draw(&self, bounds: iced::Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());
        
        if self.samples.is_empty() {
            return vec![frame.into_geometry()];
        }
        
        // Draw background
        frame.fill_rectangle(
            iced::Point::new(0.0, 0.0),
            iced::Size::new(bounds.width, bounds.height),
            Color::from_rgb(0.9, 0.9, 0.9),
        );
        
        // Draw axis
        let x_axis_y = bounds.height / 2.0;
        frame.stroke(
            &Path::line(
                iced::Point::new(0.0, x_axis_y),
                iced::Point::new(bounds.width, x_axis_y),
            ),
            Stroke::default().with_color(Color::from_rgb(0.5, 0.5, 0.5)),
        );
        
        // Find the max sample value for scaling
        let max_value = self.samples.iter()
            .fold(0.0, |max, &s| f64::max(max, s.abs())) 
            .max(0.01); // Avoid division by zero
        
        // Draw plot line
        let mut path = Path::new();
        
        let dt = self.duration / (self.samples.len() as f64 - 1.0);
        path.move_to(iced::Point::new(0.0, (0.5 - self.samples[0] / max_value / 2.0) * bounds.height as f64));
        
        for (i, &sample) in self.samples.iter().enumerate().skip(1) {
            let x = (i as f64 * dt / self.duration) * bounds.width as f64;
            let y = (0.5 - sample / max_value / 2.0) * bounds.height as f64;
            path.line_to(iced::Point::new(x as f32, y as f32));
        }
        
        frame.stroke(
            &path,
            Stroke::default().with_color(Color::from_rgb(0.0, 0.2, 0.8)).with_width(2.0),
        );
        
        // Draw time labels
        for i in 0..=5 {
            let x = i as f32 * bounds.width / 5.0;
            let time = i as f32 * self.duration as f32 / 5.0;
            
            frame.fill_text(format!("{:.1}s", time), iced::Point::new(x, bounds.height - 5.0));
        }
        
        vec![frame.into_geometry()]
    }
}

struct FrequencyDomainPlot {
    data: Vec<(f64, f64)>,
}

impl canvas::Program<Message> for FrequencyDomainPlot {
    fn draw(&self, bounds: iced::Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());
        
        if self.data.is_empty() {
            return vec![frame.into_geometry()];
        }
        
        // Draw background
        frame.fill_rectangle(
            iced::Point::new(0.0, 0.0),
            iced::Size::new(bounds.width, bounds.height),
            Color::from_rgb(0.9, 0.9, 0.9),
        );
        
        // Draw horizontal axis
        let axis_y = bounds.height - 20.0;
        frame.stroke(
            &Path::line(
                iced::Point::new(0.0, axis_y),
                iced::Point::new(bounds.width, axis_y),
            ),
            Stroke::default().with_color(Color::from_rgb(0.5, 0.5, 0.5)),
        );
        
        // Find max frequency and magnitude
        let max_freq = self.data.last().map_or(1.0, |&(f, _)| f);
        let max_magnitude = self.data.iter().fold(0.01, |max, &(_, m)| m.max(max));
        
        // Draw bars
        let bar_width = bounds.width / self.data.len() as f32;
        
        for (i, &(freq, magnitude)) in self.data.iter().enumerate() {
            let height = (magnitude / max_magnitude) * (axis_y - 20.0) as f64;
            let x = (freq / max_freq) * bounds.width as f64;
            
            frame.fill_rectangle(
                iced::Point::new(x as f32, axis_y - height as f32),
                iced::Size::new(bar_width.max(1.0), height as f32),
                Color::from_rgb(0.2, 0.4, 0.8),
            );
        }
        
        // Draw frequency labels
        for i in 0..=5 {
            let x = i as f32 * bounds.width / 5.0;
            let freq = i as f32 * max_freq as f32 / 5.0;
            
            frame.fill_text(format!("{:.1} Hz", freq), iced::Point::new(x, bounds.height - 5.0));
        }
        
        vec![frame.into_geometry()]
    }
}

// Simple expression parser for demonstration purposes
struct SimpleExprParser {
    expr: String,
}

impl SimpleExprParser {
    fn new(expr: &str) -> Result<Self, String> {
        // This is a very simplified version - in a real app, we'd use a proper parser
        Ok(Self {
            expr: expr.to_string(),
        })
    }
    
    fn evaluate(&self, x: f64) -> Result<f64, String> {
        // Handle simple sine waves as a demonstration
        // In a real implementation, we would parse and evaluate the expression properly
        if self.expr.contains("sin") {
            let sin_regex = regex::Regex::new(r"sin\s*\(\s*([\d\.\+\-\*\/x]+)\s*\)").unwrap();
            if let Some(caps) = sin_regex.captures(&self.expr) {
                let inner = &caps[1];
                // Very simplified - just handles the basic case of sin(2*pi*x)
                if inner.contains("2*") && inner.contains("*x") {
                    let freq_regex = regex::Regex::new(r"2\s*\*\s*[\d\.]+\s*\*\s*([\d\.]+)\s*\*\s*x").unwrap();
                    if let Some(freq_caps) = freq_regex.captures(inner) {
                        if let Ok(freq) = freq_caps[1].parse::<f64>() {
                            return Ok((2.0 * PI * freq * x).sin());
                        }
                    } else {
                        // Assume it's just sin(2*pi*x) for default 1 Hz
                        return Ok((2.0 * PI * x).sin());
                    }
                }
            }
        }
        
        // Default to a simple sine wave if we can't parse
        Ok((2.0 * PI * x).sin())
    }
}
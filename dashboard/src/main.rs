use reqwest::Error;
use serde::Deserialize;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Table, Row, Cell},
    Terminal,
};
use std::{io, thread, time::Duration};
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};

// API response structure
#[derive(Deserialize, Debug)]
struct StockData {
    symbol: String,
    price: f64,
    change: f64,
    change_percent: f64,
}

// Fetch stock data from a public API
async fn fetch_stock_data(symbols: &[&str]) -> Result<Vec<StockData>, Error> {
    let mut stock_data = Vec::new();
    for &symbol in symbols {
        // Replace with your actual stock API URL and key
        let url = format!(
            "https://api.example.com/stock/{}?apikey=HRO1E3YQSUZWD8Y6",
            symbol
        );
        let response: StockData = reqwest::get(&url).await?.json().await?;
        stock_data.push(response);
    }
    Ok(stock_data)
}

// Render the dashboard
fn render_dashboard(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    stock_data: &[StockData],
) -> Result<(), io::Error> {
    terminal.draw(|f| {
        let size = f.size();

        // Create a vertical layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Stock table
                ]
                .as_ref(),
            )
            .split(size);

        // Header block
        let header = Block::default()
            .borders(Borders::ALL)
            .title("Stock Dashboard");
        f.render_widget(header, chunks[0]);

        // Stock table
        let rows: Vec<Row> = stock_data
            .iter()
            .map(|stock| {
                Row::new(vec![
                    Cell::from(stock.symbol.clone()),
                    Cell::from(format!("{:.2}", stock.price)),
                    Cell::from(format!("{:.2}", stock.change)),
                    Cell::from(format!("{:.2}%", stock.change_percent)),
                ])
            })
            .collect();

        let table = Table::new(rows)
            .header(Row::new(vec![
                Cell::from("Symbol"),
                Cell::from("Price"),
                Cell::from("Change"),
                Cell::from("% Change"),
            ]).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .block(Block::default().borders(Borders::ALL).title("Stock Prices"))
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ]);

        f.render_widget(table, chunks[1]);
    })?;
    Ok(())
}

// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Symbols for the stocks you want to track
    let stock_symbols = vec!["AAPL", "GOOGL", "AMZN", "MSFT", "TSLA"];

    // Fetch stock data
    let stock_data = fetch_stock_data(&stock_symbols).await?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Render the dashboard
    let mut count = 0; // Example counter to break the loop after a few iterations
    loop {
        render_dashboard(&mut terminal, &stock_data)?;

        // Simulate periodic updates every 5 seconds
        thread::sleep(Duration::from_secs(5));

        count += 1;
        if count >= 10 { // Break the loop after 10 iterations
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}


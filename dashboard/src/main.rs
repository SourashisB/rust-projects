use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use plotters::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(num_args = 5)]
    symbols: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageResponse {
    #[serde(rename = "Time Series (Daily)")]
    time_series: Option<std::collections::HashMap<String, DailyData>>,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

struct Stock {
    symbol: String,
    dates: Vec<NaiveDate>,
    closes: Vec<f64>,
    start_price: f64,
    end_price: f64,
    percent_change: f64,
    high: f64,
    low: f64,
    total_volume: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let api_key = "HRO1E3YQSUZWD8Y6"; // Replace with your Alpha Vantage API key

    let mut stocks = Vec::new();
    for symbol in args.symbols {
        let stock = fetch_stock_data(&symbol, api_key).await?;
        generate_plot(&stock)?;
        stocks.push(stock);
    }

    generate_html(&stocks)?;
    open::that("dashboard.html")?;

    Ok(())
}

async fn fetch_stock_data(symbol: &str, api_key: &str) -> Result<Stock> {
    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY_ADJUSTED&symbol={}&apikey={}",
        symbol, api_key
    );

    let response = reqwest::get(&url).await?.json::<AlphaVantageResponse>().await?;
    let time_series = response.time_series.ok_or(anyhow::anyhow!("No time series data"))?;

    let mut data = Vec::new();
    for (date_str, daily_data) in time_series {
        if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            if let (Ok(close), Ok(volume)) = (daily_data.close.parse::<f64>(), daily_data.volume.parse::<f64>()) {
                data.push((date, close, volume));
            }
        }
    }

    data.sort_by(|a, b| a.0.cmp(&b.0));
    if data.len() > 7 {
        data = data.split_off(data.len() - 7);
    }

    let (dates, closes, volumes): (Vec<_>, Vec<_>, Vec<_>) = data.into_iter().unzip();

    let start_price = *closes.first().ok_or(anyhow::anyhow!("No closing prices"))?;
    let end_price = *closes.last().unwrap();
    let percent_change = (end_price - start_price) / start_price * 100.0;
    let high = closes.iter().fold(f64::MIN, |a, &b| a.max(b));
    let low = closes.iter().fold(f64::MAX, |a, &b| a.min(b));
    let total_volume: f64 = volumes.iter().sum();

    Ok(Stock {
        symbol: symbol.to_string(),
        dates,
        closes,
        start_price,
        end_price,
        percent_change,
        high,
        low,
        total_volume,
    })
}

fn generate_plot(stock: &Stock) -> Result<()> {
    let filename = format!("{}_plot.png", stock.symbol);
    let root = BitMapBackend::new(&filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let y_min = stock.closes.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = stock.closes.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_padding = (y_max - y_min) * 0.1;
    let y_range = (y_min - y_padding)..(y_max + y_padding);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} Stock Price", stock.symbol),
            ("sans-serif", 40).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..stock.dates.len() as i32 - 1, y_range)?;

    chart.configure_mesh()
        .x_labels(stock.dates.len())
        .x_label_formatter(&|x| {
            if *x >= 0 && *x < stock.dates.len() as i32 {
                stock.dates[*x as usize].format("%Y-%m-%d").to_string()
            } else {
                String::new()
            }
        })
        .y_label_formatter(&|y| format!("${:.2}", y))
        .draw()?;

    chart.draw_series(LineSeries::new(
        stock.closes.iter().enumerate().map(|(i, &c)| (i as i32, c)),
        &RED.stroke_width(2),
    ))?;

    Ok(())
}

fn generate_html(stocks: &[Stock]) -> Result<()> {
    let mut html = String::new();
    html.push_str(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Stock Dashboard</title>
    <style>
        .dashboard {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
            padding: 20px;
        }
        .stock {
            border: 1px solid #ddd;
            padding: 20px;
            border-radius: 8px;
            background: #f9f9f9;
        }
        .plot {
            width: 100%;
            height: 300px;
            object-fit: contain;
        }
        .analysis {
            margin-top: 15px;
            font-family: Arial, sans-serif;
        }
        h1 {
            text-align: center;
            color: #333;
        }
    </style>
</head>
<body>
    <h1>Stock Dashboard</h1>
    <div class="dashboard">
"#);

    for stock in stocks {
        html.push_str(&format!(
            r#"<div class="stock">
                <h2>{}</h2>
                <img class="plot" src="{}_plot.png" alt="Price Chart">
                <div class="analysis">
                    <p>Start Price: ${:.2}</p>
                    <p>End Price: ${:.2}</p>
                    <p>Change: {:.2}%</p>
                    <p>7-Day High: ${:.2}</p>
                    <p>7-Day Low: ${:.2}</p>
                    <p>Total Volume: {:.0}</p>
                </div>
            </div>"#,
            stock.symbol,
            stock.symbol,
            stock.start_price,
            stock.end_price,
            stock.percent_change,
            stock.high,
            stock.low,
            stock.total_volume
        ));
    }

    html.push_str(r#"
    </div>
</body>
</html>
"#);

    fs::write("dashboard.html", html)?;
    Ok(())
}
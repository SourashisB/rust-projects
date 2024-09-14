use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};
use tokio::net::TcpStream;

pub async fn start_cli(server: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(server).await?;
    println!("Connected to server at {}", server);
    println!("Available commands:");
    println!("  GET <key>");
    println!("  SET <key> <value>");
    println!("  DELETE <key>");
    println!("  LIST");
    println!("  exit");

    let mut stdin = BufReader::new(stdin());
    let mut stdout = stdout();

    loop {
        stdout.write_all(b"> ").await?;
        stdout.flush().await?;

        let mut input = String::new();
        stdin.read_line(&mut input).await?;

        let command = input.trim();
        if command == "exit" {
            break;
        }

        stream.write_all(command.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut response = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut response).await?;

        stdout.write_all(b"Server response: ").await?;
        stdout.write_all(response.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}
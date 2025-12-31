use async_nats::Client;
use dotenvy::dotenv;
use futures_util::StreamExt;
use std::env;
use std::error::Error;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file
    if let Err(e) = dotenv() {
        eprintln!("Warning: Could not load .env file: {}", e);
        eprintln!("Make sure you have a .env file with NATS_HOST, NATS_PORT, and NATS_TOKEN");
    }

    // Get NATS configuration
    let nats_host = env::var("NATS_HOST")
        .map_err(|e| format!("NATS_HOST environment variable is required: {}", e))?;
    let nats_port = env::var("NATS_PORT")
        .map_err(|e| format!("NATS_PORT environment variable is required: {}", e))?;
    let nats_token = env::var("NATS_TOKEN")
        .map_err(|e| format!("NATS_TOKEN environment variable is required: {}", e))?;

    let nats_url = format!("nats://{}:{}", nats_host, nats_port);

    // Connect to NATS
    let options = async_nats::ConnectOptions::new().token(nats_token);

    let client: Client = async_nats::connect_with_options(&nats_url, options).await?;
    let mut sub = client.subscribe("api").await?;

    println!("NATS Server listening on {} with authentication", nats_url);
    println!("Waiting for commands on 'api' subject...");

    while let Some(msg) = sub.next().await {
        let now = Instant::now();
        let payload = String::from_utf8_lossy(&msg.payload);
        println!("Received command: {}", payload);

        // Command processing
        let response = process_command(&payload).await;

        let elapsed = now.elapsed();
        if let Some(reply) = msg.reply {
            if let Err(e) = client
                .publish(reply, response.as_bytes().to_vec().into())
                .await
            {
                eprintln!("Failed to send response: {} ({:.2?})", e, elapsed);
            } else {
                println!("Sent response: {} ({:.2?})", response, elapsed);
            }
        } else {
            println!(
                "No reply subject provided, response: {} ({:.2?})",
                response, elapsed
            );
        }
    }

    Ok(())
}

async fn process_command(command: &str) -> String {
    match command.trim().to_lowercase().as_str() {
        "hello from python!" => "Hello from Rust server! Command received.".to_string(),
        "ping" => "pong".to_string(),
        "status" => "Server is running and ready to process commands".to_string(),
        cmd if cmd.starts_with("echo ") => {
            format!("Echo: {}", &cmd[5..])
        }
        _ => format!(
            "Unknown command: '{}'. Available commands: ping, status, echo <message>",
            command.trim()
        ),
    }
}

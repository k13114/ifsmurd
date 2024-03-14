use tokio::sync::{broadcast, watch};
use serde_json::{Value};
use tokio::io::AsyncBufReadExt;

pub async fn control_task(tx: watch::Sender<bool>, tx_broadcast: broadcast::Sender<Value>) {
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);

    loop {
        println!(
            "Press 'q' to quit, 's' to start another program, or any other key to continue..."
        );

        let mut line = String::new();
        if let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                println!("EOF (Ctrl-D in Unix, Ctrl-Z in Windows) is pressed");
                break;
            }
            let command = line.trim();
            match command {
                "q" => {
                    println!("Stopping the program for serial communication...");
                    if tx.send(false).is_err() {
                        eprintln!("Failed to send message");
                    }
                    //                    break;
                }
                "s" => {
                    println!("Starting the program for serial communication...");
                    if tx.send(true).is_err() {
                        eprintln!("Failed to send message");
                    }
                    let message : Value = serde_json::from_str(r#"{
                            "message" : "hello from control to start"
                        }"#).unwrap();
                    tx_broadcast
                        .send(message)
                        .expect("Error serde");
                }
                "p" => {
                    let message : Value = serde_json::from_str(r#"{
                            "message" : "hello from control p command"
                        }"#).unwrap();
                    tx_broadcast
                        .send(message)
                        .expect("Error serde");
                }
                _ => {
                    println!("Continuing the program...");
                }
            }
        } else {
            println!("Error reading from stdin");
            break;
        }
    }
}

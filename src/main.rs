//! A little MUD project in Rust.
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex};

mod player;
mod mob;
mod game_loop;
pub(crate) mod world;
pub(crate) mod traits;
pub(crate) mod string;
pub(crate) mod util;
use game_loop::game_loop;

use crate::player::Player;
use crate::string::sanitize::Sanitizer;
use crate::world::World;

pub(crate) static DATA_PATH: Lazy<Arc<String>> = Lazy::new(||
    Arc::new(std::env::var("RUSTROM_DATA")
        .expect("\
        HALT the press!\n\n\
        RUSTROM_DATA, path to the world data, is not set/given!\n\n\
        Either set it, or provide it on the command line."))
    );

#[derive(Debug)]
pub(crate) enum ClientState {
    EnteringName,
    EnteringPassword { name: String },
    Playing(Player),
    Logout,
}

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    let world = Arc::new(Mutex::new(World::new("rustrom").expect("ERROR: world dead or in fire?!")));

    tokio::spawn(game_loop(world.clone()));

    // Create a listener that will accept incoming connections.
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    log::info!("Server listening on 127.0.0.1:8080");

    // A broadcast channel is used to send messages to all connected clients.
    // Here, we're just broadcasting chat messages.
    let (tx, _rx) = broadcast::channel(10);
    const WHAT_DO_WE_CALL_YOU: &str = "What do we call you?: ";

    loop {
        // Wait for a new client to connect.
        let (mut socket, addr) = listener.accept().await.unwrap();
        log::info!("New connection from: {}", addr);

        // Clone the sender part of the broadcast channel for the new client.
        let tx = tx.clone();

        // Get a receiver for this client to listen for messages from others.
        let mut rx = tx.subscribe();

        // Spawn a new task to handle this client's connection.
        // This allows the server to handle multiple clients concurrently.
        tokio::spawn(async move {
            // Split the socket into a reader and a writer.
            let (reader, mut writer) = socket.split();

            // Use a BufReader for efficient line-by-line reading.
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // Send a welcome message to the new client.
            let mut state = ClientState::EnteringName;
            writer.write_all(b"Welcome to the Rust MUD!\n\nWhat do we call you?: ").await.unwrap();

            // This is the main loop for the client.
            loop {
                // `tokio::select!` allows us to wait on multiple async operations
                // and act on the first one that completes.
                tokio::select! {
                    // --- First Branch: Read input from the client ---
                    result = reader.read_line(&mut line) => {
                        if result.unwrap_or(0) == 0 {
                            log::info!("Client {} disconnected.", addr);
                            break;
                        }

                        let input = line.trim().sanitize();
                        match state {
                            ClientState::EnteringName => {

                            }
                            _ => {
                                let trimmed_line = line.trim();

                                // --- Command Parsing Logic ---
                                if trimmed_line.eq_ignore_ascii_case("quit") {
                                    // If the user types "quit", we break the loop to disconnect them.
                                    writer.write_all(b"Goodbye!\n").await.unwrap();
                                    break;
                                } else if trimmed_line.starts_with("say ") {
                                    // If the user types "say <message>", we broadcast it.
                                    let message = trimmed_line.strip_prefix("say ").unwrap();
                                    let full_message = format!("[{}] says: {}\n", addr, message);
                                    
                                    // Send the message to the broadcast channel.
                                    tx.send(full_message).unwrap();

                                } else {
                                    // For any other command, just echo it back.
                                    let response = format!("You said: {}\n", trimmed_line);
                                    writer.write_all(response.as_bytes()).await.unwrap();
                                }
                                
                                // Clear the buffer for the next line and show a prompt.
                                line.clear();
                                writer.write_all(b"> ").await.unwrap();
                            }
                        }
                    },

                    // --- Second Branch: Receive broadcast messages from other clients ---
                    result = rx.recv() => {
                        match result {
                            Ok(msg) => {
                                // If we receive a message from the broadcast channel,
                                // write it to our client's socket.
                                writer.write_all(msg.as_bytes()).await.unwrap();
                                // Also write the prompt again so the user can type.
                                writer.write_all(b"> ").await.unwrap();
                            },
                            Err(_) => {
                                // This can happen if the broadcast channel is slow.
                                // We can just ignore it for this simple example.
                            }
                        }
                    }
                }
            }
        });
    }
}

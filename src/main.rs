//! A little MUD project in Rust.
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};

mod player;
mod mob;
mod game_loop;
pub(crate) mod world;
pub(crate) mod traits;
pub(crate) mod string;
pub(crate) mod util;
mod cmd;
use game_loop::game_loop;

use crate::mob::core::IsMob;
use crate::player::access::Access;
use crate::player::save::{LoadError, Player};
use crate::string::prompt::PromptType;
use crate::string::sanitize::Sanitizer;
use crate::traits::save::DoesSave;
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
    EnteringPassword1 { name: String },
    EnteringPasswordV { name: String, pw1: String },
    Playing(Player),
    Logout(Player),
}

#[tokio::main]
async fn main() {
    const GREETING: &str = "Welcome to RustROM!";
    const PROMPT_LOGIN: &str = "What do we call you?: ";
    const PROMPT_PASSWD1: &str = "Password: ";
    const PROMPT_PASSWDV: &str = "Re-type same password: ";
    const PROMPT_PLAYING: &str = "#> ";
    const WELCOME_BACK: &str = "Welcome back!";
    const WELCOME_NEW: &str = "May your adventures be prosperous!";

    // Initialize the logger
    env_logger::init();

    let world = Arc::new(RwLock::new(World::new("rustrom").expect("ERROR: world dead or in fire?!")));

    tokio::spawn(game_loop(world.clone()));

    // Create a listener that will accept incoming connections.
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    log::info!("Server listening on 127.0.0.1:8080");

    // A broadcast channel is used to send messages to all connected clients.
    // Here, we're just broadcasting chat messages.
    let (tx, _) = broadcast::channel::<String>(16);
    
    loop {
        // Wait for a new client to connect.
        let (socket, addr) = listener.accept().await.unwrap();
        log::info!("New connection from: {}", addr);

        // Clone the sender part of the broadcast channel for the new client.
        let tx = tx.clone();

        // Get a receiver for this client to listen for messages from others.
        let mut rx = tx.subscribe();
        let world = world.clone();

        // Spawn a new task to handle this client's connection.
        // This allows the server to handle multiple clients concurrently.
        tokio::spawn(async move {
            // Split the socket into a reader and a writer.
            let (reader, mut writer) = socket.into_split();

            // Use a BufReader for efficient line-by-line reading.
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // Send a welcome message to the new client.
            let mut state = ClientState::EnteringName;
            let (greeting, login_prompt) = {
                let w = world.read().await;
                let g = w.greeting.clone().unwrap_or_else(|| GREETING.to_string());
                let p = w.prompts.get(&PromptType::Login).cloned().unwrap_or_else(|| PROMPT_LOGIN.to_string());
                (g, p)
            };
            let mut prompt = login_prompt;
            writer.write_all(format!("{}\n\n{}", greeting, prompt).as_bytes()).await.unwrap();
            let mut abrupt_dc = false;

            // This is the main loop for the client.
            loop {
                // Check if player is logging out...
                if let ClientState::Logout(mut player) = state {
                    log::info!("Player '{}' logging out.", player.name());
                    let _ = player.save().await;
                    if !abrupt_dc {
                        tell_user!(writer, "Goodbye! See you soon again!\n");
                    }
                    break;
                }

                line.clear();
                tokio::select! {
                    // --- First Branch: Read input from the client ---
                    result = reader.read_line(&mut line) => {
                        // An abrupt disconnect?
                        if result.unwrap_or(0) == 0 {
                            log::info!("Client {} disconnected.", addr);
                            if let ClientState::Playing(player) = state {
                                // Shift to logout state and re-loop…
                                abrupt_dc = true;
                                state = ClientState::Logout(player);
                                continue;
                            } else {
                                // They weren't playing - nothing to save - d/c.
                                break;
                            }
                        }

                        let input = line.trim().sanitize();
                        let old_state = std::mem::replace(&mut state, ClientState::EnteringName);
                        state = match old_state {
                            ClientState::Playing(player) => {
                                cmd::parse_and_execute(player, &world, &tx, &input, &mut writer, &prompt).await
                            },
                            ClientState::EnteringName => {
                                if input.is_empty() {
                                    tell_user!(writer, prompt);
                                    state
                                } else {
                                    log::info!("Login attempt on '{}'…", input);
                                    prompt = get_prompt!(world, PromptType::Password1, PROMPT_PASSWD1);
                                    tell_user!(writer, prompt);
                                    ClientState::EnteringPassword1 { name: input.to_string() }
                                }
                            },
                            ClientState::EnteringPassword1{ name } => {
                                match Player::load(&name, &input).await {
                                    Ok(save) => {
                                        log::info!("'{}' successfully logged in.", name);
                                        let (msg, p) = {
                                            let w = world.read().await;
                                            let msg = w.welcome_back.clone().unwrap_or_else(|| WELCOME_BACK.to_string());
                                            let p = get_g_prompt!(w, PromptType::Playing, PROMPT_PLAYING);
                                            (msg, p)
                                        };
                                        prompt = p;
                                        tell_user!(writer, format!("{}\n\n{}", msg, prompt));
                                        ClientState::Playing(save)
                                    },
                                    Err(LoadError::NoSuchSave) => {
                                        prompt = get_prompt!(world, PromptType::PasswordV, PROMPT_PASSWDV);
                                        tell_user!(writer, prompt);
                                        ClientState::EnteringPasswordV { name, pw1: input }
                                    },
                                    Err(e) => {
                                        log::warn!("Failed login attempt for '{}': {:?}", name, e);
                                        prompt = get_prompt!(world, PromptType::Login, PROMPT_LOGIN);
                                        tell_user_p!(writer, prompt, "Invalid name and/or password.");
                                        ClientState::EnteringName
                                    }
                                }
                            },
                            ClientState::EnteringPasswordV{ name, pw1 } => {
                                if input == pw1 {
                                    let mut player = Player::new(&name);
                                    if player.set_passwd(input).await.is_ok() {
                                        log::info!("New save being created for '{}'…", name);
                                        prompt = get_prompt!(world, PromptType::Playing, PROMPT_PLAYING);
                                        player.set_access(Access::default());
                                        let save_err = player.save().await;
                                        if save_err.is_ok() {
                                            let msg = {world.read().await.welcome_new.clone().unwrap_or_else(|| WELCOME_NEW.to_string())};
                                            tell_user_p!(writer, prompt, msg);
                                            ClientState::Playing(player)
                                        } else {
                                            // Some strange error happened with save...
                                            // Notify user and "gracefully" disconnect them.
                                            log::error!("Fatal error during save attempt of player '{}'! {:?}", name, save_err);
                                            tell_user!(writer, "\
                                                    A server error occured during character creation!\n\n\
                                                    This could be due high server load or other reasons. Try again a little later, but meanwhile \
                                                    please, notify the owner of this MUD via email or other means!");
                                            break;
                                        }
                                    } else {
                                        tell_user!(writer, "Given password is either too weak or a variant of it has been found in HIBP!\nPlease, choose a different password: ");
                                        ClientState::EnteringPassword1 { name }
                                    }
                                } else {
                                    tell_user!(writer, "Passwords do not match. Please choose a password: ");
                                    ClientState::EnteringPassword1 { name }
                                }
                            },
                            ClientState::Logout(player) => ClientState::Logout(player)// needed, even though handled at top of loop earlier.
                        };
                    },

                    // --- Second Branch: Receive broadcast messages from other clients ---
                    result = rx.recv() => {
                        if let ClientState::Playing(_) = &state {
                            if let Ok(msg) = result {
                                // If we receive a message from the broadcast channel,
                                // write it to our client's socket.
                                writer.write_all(msg.as_bytes()).await.unwrap();
                                // Also write the prompt again so the user can type.
                                writer.write_all(b"> ").await.unwrap();
                            }
                        }
                    }
                }
            }
        });
    }
}

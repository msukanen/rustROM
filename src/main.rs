//! A little MUD project in Rust.
use std::{collections::HashMap, ops::Deref, sync::Arc};
use clap::Parser;
use once_cell::sync::OnceCell;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    sync::{broadcast, RwLock}
};

mod player;
mod mob;
mod game_loop;
use game_loop::game_loop;
pub mod world;
pub mod traits;
pub mod string;
pub mod util;
mod cmd;

use crate::{cmd::CommandCtx, mob::core::IsMob, traits::Description, util::{help::Help, ClientState}};
use crate::player::{access::Access, LoadError, Player};
use crate::string::{prompt::PromptType, sanitize::Sanitizer};
use crate::traits::save::DoesSave;
use crate::world::World;

pub struct ImmutablePath; impl ImmutablePath {
    pub fn set(path: impl Into<String>) {
        DATA.set(path.into()).expect("FFS!");
    }
}
impl Deref for ImmutablePath {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        DATA.get().expect("OOF")
    }
}
static DATA: OnceCell<String> = OnceCell::new();
pub (crate) static DATA_PATH: ImmutablePath = ImmutablePath;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A RustROM MUD engine.",
    after_help = "\
Note:   The data path can also be set using the RUSTROM_DATA environment\n\
\tvariable, for example:\n\n\
Usage:  RUSTROM_DATA=/path/to/data rustrom [OPTIONS]
        "
)]
struct CmdLineArgs {
    #[arg(short, long, default_value = "8080")]
    port: u32,
    #[arg(long, default_value = "0.0.0.0")]
    host_listen_addr: String,
    #[arg(long, default_value = "rustrom")]
    world: String,
    #[arg(long, env = "RUSTROM_DATA", default_value = "data")]
    data_path: String,
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

    let args = CmdLineArgs::parse();
    let _ = DATA.set(args.data_path);

    // Initialize the logger
    env_logger::init();

    // Load the world ...
    let world = Arc::new(RwLock::new({
        let w = World::new(&args.world).await.expect("ERROR: world dead or in fire?!");
        w.validate().await.expect(&format!("Error validating {}", "rustrom.world"))
    }));{
        log::info!("Connecting dots …");
        let mut w = world.write().await;
        let mut collected_rooms_to_add = HashMap::new();
        for area_arc in w.areas.values_mut() {
            let mut a = area_arc.write().await;
            log::info!("… processing area '{}'", a.id);
            a.parent = Arc::downgrade(&world);

            for (room_stem, room_arc) in &a.rooms {
                let mut r = room_arc.write().await;
                log::info!("… making ↑ connect for room '{}' (a.k.a. '{}')", r.id(), r.title());
                r.parent = Arc::downgrade(area_arc);
                collected_rooms_to_add.insert(room_stem.clone(), room_arc.clone());
            }
        }
        w.rooms = collected_rooms_to_add;
    }

    // Load help files ...
    world.write().await.help = Help::load_all().await.expect("Oopsie - we're helpless - no help available?!");

    tokio::spawn(game_loop(world.clone()));

    // Create a listener that will accept incoming connections.
    let listen_on = format!("{}:{}", args.host_listen_addr, args.port);
    let listener = TcpListener::bind(&listen_on).await.unwrap();
    log::info!("Server listening on {}", listen_on);

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
            tell_user!(&mut writer, "{}\n\n{}", greeting, &login_prompt);
            let mut abrupt_dc = false;

            // This is the main loop for the client.
            loop {
                // Check if player is logging out...
                if let ClientState::Logout = &state {
                    let mut w = world.write().await;
                    if let Some(p) = w.players.remove(&addr.ip()) {
                        let mut p = p.write().await;
                        log::info!("Player '{}' logging out.", p.id());
                        if let Err(e) = p.save().await {
                            log::error!("Error saving '{}'! {:?}", p.id(), e);
                        }
                        if !abrupt_dc {
                            tell_user!(&mut writer, "<c cyan>Goodbye! See you soon again!</c>\n");
                        }
                    }
                    break;
                }

                line.clear();
                tokio::select! {
                    // --- First Branch: Read input from the client ---
                    result = reader.read_line(&mut line) => {
                        // An abrupt disconnect?
                        if result.unwrap_or(0) == 0 {
                            log::info!("Client {} disconnected abruptly.", addr);

                            match &state {
                                ClientState::Playing |
                                ClientState::Editing {..}
                                => {// Shift to logout state and re-loop…
                                    abrupt_dc = true;
                                    state = ClientState::Logout;
                                    continue
                                },
                                _ => break// They weren't playing - nothing to save - d/c.
                            }
                        }

                        let input = line.trim().sanitize();

                        state = match state {
                            ClientState::EnteringName => {
                                if input.is_empty() {
                                    tell_user!(&mut writer, &login_prompt);
                                    state
                                } else {
                                    log::info!("Login attempt on '{}'…", input);
                                    tell_user!(&mut writer, get_prompt!(world, PromptType::Password1, PROMPT_PASSWD1));
                                    ClientState::EnteringPassword1 { name: input.to_string() }
                                }
                            },
                            ClientState::EnteringPassword1{ name } => {
                                match Player::load(&name, &input, &addr).await {
                                    Ok(mut save) => {
                                        log::info!("'{}' successfully logged in.", name);
                                        let (msg, pr) = {
                                            let mut w = world.write().await;
                                            save.erase_states(ClientState::Playing);
                                            let pr = save.prompt().await;
                                            let p = Arc::new(RwLock::new(save));
                                            w.players.insert(addr.ip(), p.clone());
                                            (w.welcome_back.clone().unwrap_or_else(|| WELCOME_BACK.to_string()), pr)
                                        };
                                        tell_user!(&mut writer, "{}\n\n{}", msg, pr);
                                        ClientState::Playing
                                    },
                                    Err(LoadError::NoSuchSave) => {
                                        tell_user!(&mut writer, "{}", get_prompt!(world, PromptType::PasswordV, PROMPT_PASSWDV));
                                        ClientState::EnteringPasswordV { name, pw1: input }
                                    },
                                    Err(e) => {
                                        log::warn!("Failed login attempt for '{}': {:?}", name, e);
                                        tell_user!(&mut writer, "Invalid name and/or password.\n\n{}", get_prompt!(world, PromptType::Login, PROMPT_LOGIN));
                                        ClientState::EnteringName
                                    }
                                }
                            },
                            ClientState::EnteringPasswordV{ name, pw1 } => {
                                if input == pw1 {
                                    let mut player = Player::new(&name);
                                    if player.set_passwd(input).await.is_ok() {
                                        log::info!("New save being created for '{}'…", name);
                                        let prompt = get_prompt!(world, PromptType::Playing, PROMPT_PLAYING);
                                        player.set_access(Access::default());
                                        let save_err = player.save().await;
                                        if save_err.is_ok() {
                                            let msg = {world.read().await.welcome_new.clone().unwrap_or_else(|| WELCOME_NEW.to_string())};
                                            tell_user!(&mut writer, "{}\n{}", msg, prompt);
                                            let p = Arc::new(RwLock::new(player));
                                            world.write().await.players.insert(addr.ip(), p.clone());
                                            p.write().await.erase_states(ClientState::Playing)
                                        } else {
                                            // Some strange error happened with save...
                                            // Notify user and "gracefully" disconnect them.
                                            log::error!("Fatal error during save attempt of player '{}'! {:?}", name, save_err);
                                            tell_user!(&mut writer, "\
                                                    A server error occured during character creation!\n\
                                                    \n\
                                                    This could be due high server load or other reasons. \
                                                    Try again a little later, but meanwhile please, notify \
                                                    the owner of this MUD via email or other means!");
                                            break;
                                        }
                                    } else {
                                        tell_user!(&mut writer, "\
                                                Given password is either too weak or a variant of it has been found in HIBP!\n\
                                                Please, choose a different password: ");
                                        ClientState::EnteringPassword1 { name }
                                    }
                                } else {
                                    tell_user!(&mut writer, "Passwords do not match.\n\nPlease choose a password: ");
                                    ClientState::EnteringPassword1 { name }
                                }
                            },
                            _ => {
                                let p = {
                                    let w = world.read().await;
                                    w.players.get(&addr.ip()).cloned()
                                };
                                let prompt: String;
                                if let Some(p) = p {
                                    let ctx = CommandCtx {
                                        player: p.clone(),
                                        world: &world,
                                        tx: &tx,
                                        args: &input,
                                        writer: &mut writer,
                                        };
                                    state = cmd::parse_and_execute(ctx).await;//state, p.clone(), &world, &tx, &input, &mut writer).await;
                                    prompt = p.read().await.prompt().await;
                                } else {
                                    // player a goner?!
                                    abrupt_dc = true;
                                    state = ClientState::Logout;
                                    continue;
                                }
                                tell_user!(&mut writer, prompt);
                                state
                            },
                        };
                    },

                    // --- Second Branch: Receive broadcast messages from other clients ---
                    result = rx.recv() => {
                        if let ClientState::Playing = &state {
                            if let Ok(msg) = result {
                                // If we receive a message from the broadcast channel, write it to our client.
                                let w = world.read().await;
                                if let Some(p) = w.players.get(&addr.ip()) {
                                    tell_user!(&mut writer, "{}{}", msg, p.read().await.prompt().await);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

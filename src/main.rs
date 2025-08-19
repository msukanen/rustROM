//! A little MUD project in Rust.
use std::{collections::{HashMap, HashSet}, ops::Deref, sync::Arc};
use clap::Parser;
use once_cell::sync::OnceCell;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    sync::{broadcast, RwLock}
};

mod player;
mod mob;
mod game_loop;  use game_loop::game_loop;
mod io;         use io::io_loop;
pub mod world;
pub mod traits;
pub mod string;
pub mod util;
mod cmd;

use crate::{cmd::{translocate::translocate, CommandCtx}, mob::core::IsMob, string::WordSet, traits::Description, util::{help::Help, BroadcastMessage, ClientState}, world::room::find_nearby_rooms};
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
#[cfg(test)]
pub(crate) static DATA: OnceCell<String> = OnceCell::new();
#[cfg(not(test))]
static DATA: OnceCell<String> = OnceCell::new();
pub(crate) static DATA_PATH: ImmutablePath = ImmutablePath;

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
    #[arg(short, long, default_value = "8080")]                 port: u32,
    #[arg(long, default_value = "0.0.0.0")]                     host_listen_addr: String,
    #[arg(long, default_value = "rustrom")]                     world: String,
    #[arg(long, env = "RUSTROM_DATA", default_value = "data")]  data_path: String,
    #[arg(long)]                                                bootstrap_url: Option<String>,
}

#[tokio::main]
async fn main() {
    const GREETING: &str = "Welcome to RustROM!";
    const PROMPT_LOGIN: &str = "What do we call you?: ";
    const PROMPT_PASSWD1: &str = "Password: ";
    const PROMPT_PASSWDV: &str = "Re-type same password: ";
    const WELCOME_BACK: &str = "Welcome back!";
    const WELCOME_NEW: &str = "May your adventures be prosperous!";

    let args = CmdLineArgs::parse();
    let _ = DATA.set(args.data_path);

    // Initialize the logger
    env_logger::init();

    let bad_words: Arc<RwLock<WordSet>> = Arc::new(RwLock::new(HashSet::new()));

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

    // Bootstrap helps, if needed ...
    Help::bootstrap(args.bootstrap_url).await.expect("Bootstrapping failed?!");
    // Load help files ...
    let (help_core, help_aliases) = Help::load_all().await.expect("Oopsie - we're helpless - no help available?!");
    world.write().await.help = help_core;
    world.write().await.help_aliased = help_aliases;

    tokio::spawn(game_loop(world.clone()));
    tokio::spawn(io_loop(world.clone(), bad_words.clone()));

    // Create a listener that will accept incoming connections.
    let listen_on = format!("{}:{}", args.host_listen_addr, args.port);
    let listener = TcpListener::bind(&listen_on).await.unwrap();
    log::info!("Server listening on {}", listen_on);

    // A broadcast channel is used to send messages to all connected clients.
    // Here, we're just broadcasting chat messages.
    let (tx, _) = broadcast::channel::<BroadcastMessage>(16);
    
    loop {
        // Wait for a new client to connect.
        let (socket, addr) = listener.accept().await.unwrap();
        log::info!("New connection from: {}", addr);

        // Clone the sender part of the broadcast channel for the new client.
        let tx = tx.clone();

        // Get a receiver for this client to listen for messages from others.
        let mut rx = tx.subscribe();
        let world = world.clone();
        let bad_words = bad_words.clone();

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
                    if let Some(p) = w.players_by_sockaddr.remove(&addr) {
                        // drop the named mapping here as it's not needed for logout.
                        w.players.remove(p.read().await.id());
                        w.players_to_logout.push(p);
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
                                    if let Err(LoadError::InvalidName) = Player::load_is_possible(bad_words.clone(), &input).await {
                                        tell_user!(&mut writer, "Name '{}' is reserved, please try another.\n\n{}", input, get_prompt!(world, PromptType::Login, PROMPT_LOGIN));
                                        ClientState::EnteringName
                                    } else {
                                        tell_user!(&mut writer, get_prompt!(world, PromptType::Password1, PROMPT_PASSWD1));
                                        ClientState::EnteringPassword1 { name: input.to_string() }
                                    }
                                }
                            },
                            ClientState::EnteringPassword1{ name } => {
                                match Player::load(&name, &input, &addr).await {
                                    Ok(mut save) => {
                                        let mut translocated = false;
                                        log::info!("'{}' successfully logged in.", name);
                                        let (msg, prompt) = {
                                            save.erase_states(ClientState::Playing);
                                            let p = Arc::new(RwLock::new(save));

                                            let location = p.read().await.location.clone();
                                            let root_room = world.read().await.root.room.clone();
                                            if !world.read().await.rooms.contains_key(&location) {
                                                let pg = p.read().await;
                                                log::warn!("Player '{}' location '{}' invalid. Translocating to safety of '{}'.", pg.id(), location, root_room);
                                                translocated = true;
                                            }
                                            let source = location;
                                            // Relocate player in case their saved location has evaporated...
                                            let _ = translocate(&world, Some(source), root_room, p.clone()).await;
                                            let mut w = world.write().await;
                                            w.players_by_sockaddr.insert(addr.clone(), p.clone());
                                            let prompt = {
                                                let pl = p.read().await;
                                                w.players.insert(pl.id().into(), p.clone());
                                                pl.prompt().await
                                            };
                                            (w.welcome_back.clone().unwrap_or_else(|| WELCOME_BACK.to_string()), prompt)
                                        };
                                        tell_user!(&mut writer, "{}\n\n{}{}",
                                            msg,
                                            if translocated {
                                                format!("You notice something... odd - you're not where you were before... But such happens, apparently.\n\n")
                                            } else {"".into()},
                                            prompt,
                                        );
                                        ClientState::Playing
                                    },
                                    Err(LoadError::InvalidName) => {
                                        tell_user!(&mut writer, "Name '{}' is reserved, please try another.\n\n{}", name, get_prompt!(world, PromptType::Login, PROMPT_LOGIN));
                                        ClientState::EnteringName
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
                                        player.set_access(Access::default());
                                        player.location = world.read().await.root.room.clone();
                                        let save_err = player.save().await;
                                        if save_err.is_ok() {
                                            let msg = {world.read().await.welcome_new.clone().unwrap_or_else(|| WELCOME_NEW.to_string())};
                                            let p = Arc::new(RwLock::new(player));
                                            let root_room = world.read().await.root.room.clone();
                                            let _ = translocate(&world, None, root_room, p.clone()).await;
                                            let (p_id, prompt, state) = {
                                                let mut pl = p.write().await;
                                                let p_id = pl.id().to_string();
                                                let prompt = pl.prompt().await;
                                                log::info!("New player '{}' instantiated and translocated to '{}'.", p_id, &pl.location);
                                                (p_id, prompt, pl.erase_states(ClientState::Playing))
                                            };
                                            {
                                                let mut w = world.write().await;
                                                w.players_by_sockaddr.insert(addr.clone(), p.clone());
                                                w.players.insert(p_id, p.clone());
                                            }
                                            tell_user!(&mut writer, "{}\n{}", msg, prompt);
                                            state
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
                                    w.players_by_sockaddr.get(&addr).cloned()
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
                        /*
                        We handle *majority* of broadcast messages in Playing state only, which avoids
                        e.g. editor modes from being disturbed.
                        */
                        if let ClientState::Playing = &state {
                            if let Ok(msg) = result {
                                // If we receive a message from the broadcast channel, write it to our client.
                                let w = world.read().await;
                                if let Some(p) = w.players_by_sockaddr.get(&addr) {
                                    let p = p.read().await;
                                    match msg {
                                        BroadcastMessage::Say { room_id, message, from_player, .. } => {
                                            if p.location == room_id && p.id() != from_player {
                                                tell_user!(&mut writer, "{}{}", message, p.prompt().await);
                                            }
                                        },
                                        BroadcastMessage::Shout { room_id, message, from_player } => {
                                            let nearby = find_nearby_rooms(&world, &room_id, 2).await;
                                            for r_id in nearby {
                                                if p.location == r_id && p.id() != from_player {
                                                    tell_user!(&mut writer, "{}{}", message, p.prompt().await);
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

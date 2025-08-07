# TODO

## Known Secrets

Implement some sort of HashMap about "secrets" known per player, e.g.:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    ...
    known_secrets: HashMap<
        String, // ID
        String  // Details, etc.
    >;
    ...
}
```

## Hot-reload

Breakdown of the wizardry it would require:

### 1\. Listening for the Signal

First, listen for the `SIGUSR1` signal without blocking the server. The `tokio::signal` module is designed for exactly this. In `main()` function, spawn a separate task whose only job is to wait for the signal.

```rust
// In main() function
tokio::spawn(async move {
    let mut stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()).unwrap();
    loop {
        stream.recv().await;
        log::info!("SIGUSR1 received! Triggering hot-reload...");
        // When the signal is received, trigger the reload logic.
        // This could be done by sending a message over another channel.
    }
});
```

### 2\. Storing File Hashes

To know what's changed, store the state of the files from the last load. The perfect place for this is inside `World` struct. Add a new field:

```rust
// In your World struct
#[derive(Debug)]
pub struct World {
    // ... other fields ...
    #[serde(skip)] // This is runtime state, not saved
    file_hashes: HashMap<String, u64>,
}
```

When loading the world, hash each file and populate this map.

### 3\. The Hot-Reload Logic

When `SIGUSR1` signal is received, trigger "hot-reload" function. This function would:

1. Get an exclusive `write` lock on the `World`.
2. Walk through the `data/areas/` and `data/rooms/` directories.
3. For each file, calculate its current hash.
4. Compare the new hash to the one stored in `world.file_hashes`.
5. **If the hash is different:**
      * Call the appropriate `Area::load(...)` or `Room::load(...)` function to load the new version into a temporary variable.
      * Handle player relocation, when/if need (check if anyone's location is now invalid and moving them to a safe spot).
      * Replace the old `Arc<RwLock<...>>` in the `World`'s `HashMap` with the new one.
      * Update the `world.file_hashes` map with the new hash.
6. Release the `write` lock.

## GIT

Implement some means to pull/push areas and such from/to GitHub.

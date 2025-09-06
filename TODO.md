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

Breakdown of what it'd require:

### 1\. The Monkeys Are Listening…

Listen for e.g. `SIGUSR1` signal without blocking the server. `tokio::signal` module is designed for this.
Spawn a separate task whose only job is to wait for the signal.

```rust
// In main() function
tokio::spawn(async move {
    let mut stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()).unwrap();
    loop {
        stream.recv().await;
        // do stuff
    }
});
```

### 2\. The Hot-Reload Logic

When `SIGUSR1` signal is received, trigger "hot-reload" function.

1. `write` lock the `World`.
2. Walk through world, areas, rooms, help files, etc.
3. calculate their hashes, and if the hash is different than ye olde:
      * Handle player relocation, when/if need (check if anyone's location is now invalid and moving them to a safe spot).
      * Update the `world.file_hashes` map with the new hash.

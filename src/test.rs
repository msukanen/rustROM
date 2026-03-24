//! Testing stuff
#[cfg(test)]
mod async_client_server {

#[macro_export]
macro_rules! async_client_for_tests {
    ($addr:ident, $($cmd:expr),*) => {
            tokio::spawn(async move {
            let (reader, mut writer) = TcpStream::connect($addr).await.unwrap().into_split();
            log::debug!("client_task: connected.");
            let mut reader = BufReader::new(reader);
            let mut buffer = String::new();
            // Send the cmd(s)…
            $(writer.write_all(format!("{}\n", $cmd).as_bytes()).await.unwrap();)*
            // Now, read all the output until the server closes the connection.
            reader.read_to_string(&mut buffer).await.unwrap();
            log::debug!("client_tast: server response|→{}←|", buffer);
            buffer
        })
    };
}

#[macro_export]
macro_rules! async_server_for_tests {
    ($w:ident, $listener:ident, $tx:ident, $addr:ident, $p:ident, $num_cmd:literal) => {
        tokio::spawn(async move {
            let (server_socket, client_addr) = $listener.accept().await.unwrap();
            log::debug!("server_task: connection from {:?}", $addr);
            $w.write().await.players_by_sockaddr.insert(client_addr, $p);
            let (server_reader, mut server_writer) = server_socket.into_split();
            let mut server_reader = BufReader::new(server_reader);
            let mut line = String::new();
            log::debug!("server_task: player_arc?");
            let player_arc = $w.read().await.players_by_sockaddr.get(&client_addr).unwrap().clone();
            log::debug!("server_task: player_arc ok");
            player_arc.write().await.push_state(ClientState::Playing);
            player_arc.write().await.location = "nowhere-at-all".into();
            // we don't care about translocate result here!
            let _ = crate::translocate(&$w, None, "void".into(), player_arc.clone()).await;

            for i in 0..$num_cmd {
                line.clear();
                server_reader.read_line(&mut line).await.unwrap();
                let ctx = CommandCtx {
                    player: player_arc.clone(),
                    state: player_arc.read().await.state(),
                    world: &$w,
                    tx: &$tx,
                    args: &line.trim(),
                    writer: &mut server_writer };
                crate::cmd::parse_and_execute(ctx).await;
                log::info!("server_task: client cmd#{} \"{}\"", i+1, line.trim());
            }
        })
    }
}

#[macro_export]
macro_rules! player_and_listener_for_tests {
    () => {{
            let p = Arc::new(RwLock::new(Player::new("ani")));
            p.write().await.location = "void".into();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let (tx, _) = broadcast::channel::<Broadcast>(1);
            (p, listener, addr, tx)
    }}
}

/*END*/}

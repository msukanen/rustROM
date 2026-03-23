# RustROM

A MUD engine (to be).

Too much in flux at the moment to properly document anything yet.

## Multi-threading?

Yes, with `tokio`. At the moment there's some *primary* threads always
chugging along:

* main
* game_loop
* io_loop

### **main** Thread

Deals directly with e.g.:

* incoming connections
* logout/disconnects

#### Sub-threads; one per client

Each incoming connecting gets its own thread that deals (in)directly with e.g.:

* player loading
* command dispatch
* message broadcasting

### game_loop thread

Ticks the world...

### io_loop thread

Takes care of timed/reactive disk I/O, etc.…

* saves (and purges) disconnected players.
* bootstraps "bad names" word list from github, if/when needed.
* auto-saves online players.
* runtime lost-and-found items collecting.
* ... other things to come.

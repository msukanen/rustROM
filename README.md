# RustROM

A MUD engine (to be).

Too much in flux at the moment to properly document anything yet.

## Multi-threading?

Yes, with `tokio`. At the moment there's three primary threads constantly
chugging along:

* main
* game_loop
* io_loop

### main thread

Deals with…

* incoming connections and
* message broadcasting.

### game_loop thread

Ticks the world...

### io_loop thread

Takes care of some async I/O ...

* saves (and purges) disconnected players.
* bootstraps "bad names" word list from github, if/when needed.
* auto-saves online players.
* ... other things to come.

# Abandoned!

This project has been Abandoned before beeing (properly) finished (there's no frontend xd). The reason for this is that projects like [cargo-tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html), [cargo-graph](https://github.com/kbknapp/cargo-graph), [cargo-depgraph](https://github.com/jplatte/cargo-depgraph) exist and choose a better strategies to show dependencies. Also, the graph database I chose will be in "maintanence mode" (end-of-life) soon (redisgraph). Sooo, in the current state:

- pre-processor works
  - takes cargo crates dump and creates a redisgraph database with inter-connected dependency versions
  - uses a shit-tone of RAM (crates.io has a lot of packages), so setup at least 32GB swap beforehand!
- api works
  - uses the database to traverse dependencies for a package version (and caches the traversed results)
  - Example usage of the API: `https://localhost:50001/api/v1/cargo/crate/v/:version_id/traverse` (every package version has unique ID, non-dependent on the package)
- frontend
  - nonexistent lol
- use this repo as more of an example on how to work with:
  - cargo crates data dumps
  - rust's axum web server
  - rust + redisgraph (even tho it's EOL)

# Dev Setup

## Cargo data dump

### Setup

- Download latest data dump: https://static.crates.io/db-dump.tar.gz
- Go into `./data-dumps/cargo/`
  - Place the downloaded dump here and run `tar -xf db-dump.tar.gz`
  - Rename the extracted folder from it's original name to `db-dump`
  - `docker run -it --name deps-graph-cargo-db-dump-container -d -p 7501:5432 -v "$(pwd)/db-dump:/var/lib/postgresql/db-dump" -v "$(pwd)/db-container-data/:/var/lib/postgresql/data" -e POSTGRES_PASSWORD=c4rg0DUmP -e POSTGRES_USER=dumpuser -e POSTGRES_DB=dumpdb -e PGDATA=/var/lib/postgresql/data/pgdata postgres:15.3-alpine` (on Windows replace `pwd` with `pwd -W`)
  - `docker exec -it deps-graph-cargo-db-dump-container bash`
  - `cd /var/lib/postgresql/db-dump`
  - `psql dumpdb -U dumpuser < schema.sql`
  - `psql dumpdb -U dumpuser < import.sql`

### Continue after shutdown

- `docker start deps-graph-cargo-db-dump-container`

### Shutdown / cleanup

- Go into `./data-dumps/cargo/`
  - `docker ps` -- get id
  - `docker stop {id}`
  - `docker rm {id}`
  - `docker volume ls` -- get id
  - `docker volume rm {id}`
  - `rm -rf db-container-data db-dump`

## Redisgraph

### Setup

- Go into `./data-dump/redisgraph/`
  - `docker run -it --name deps-graph-redisgraph -d -p 7500:6379 -v "$(pwd)/db-container-data/:/data" redislabs/redisgraph:2.12.1` (on Windows, replace `pwd` with `pwd -W`)
  - If you want to try out running in constrained enviroment, add following flags: `--memory="1g" --memory-swap="9g"` before `-d` flag in the previous command

### Continue after shutdown

- `docker start deps-graph-redisgraph`

### Shutdown / cleanup

- Go into `./data-dumps/redisgraph/`
  - `docker ps` -- get id
  - `docker stop {id}`
  - `docker rm {id}`
  - `docker volume ls` -- get id
  - `docker volume rm {id}`
  - `rm -rf db-container-data`

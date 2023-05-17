# Dev Setup

## Docker

### Cargo data dump

#### Setup

- Download latest data dump: https://static.crates.io/db-dump.tar.gz
- Go into `./data-dumps/cargo/`
  - Place the downloaded dump here and run `tar -xf db-dump.tar.gz`
  - Rename the extracted folder from it's original name to `db-dump`
  - `docker run -it --name deps-graph-cargo-db-dump-container -d -p 7501:5432 -v "$(pwd)/db-dump:/var/lib/postgresql/db-dump" -v "$(pwd)/db-container-data/:/var/lib/postgresql/data" -e POSTGRES_PASSWORD=c4rg0DUmP -e POSTGRES_USER=dumpuser -e POSTGRES_DB=dumpdb -e PGDATA=/var/lib/postgresql/data/pgdata postgres:15.3-alpine`
  - `docker exec -it deps-graph-cargo-db-dump-container bash`
  - `cd /var/lib/postgresql/db-dump`
  - `psql dumpdb -U dumpuser < schema.sql`
  - `psql dumpdb -U dumpuser < import.sql`

#### Shutdown / cleanup

- Go into `./data-dumps/cargo/`
  - `docker ps` -- get id
  - `docker stop {id}`
  - `docker rm {id}`
  - `docker volume ls` -- get id
  - `docker volume rm {id}`
  - `rm -rf db-container-data`

### Redisgraph

#### Startup

- Go into `./data-dump/redisgraph/`
  - `docker run -it --name deps-graph-redisgraph -d -p 7500:6379 -v "$(pwd)/db-container-data/:/data" redislabs/redisgraph:2.12.1`
  - If you want to try out running in constrained enviroment, add following flags: `--memory="1g" --memory-swap="9g"` before `-d` flag in the previous command

#### Shutdown / cleanup

- Go into `./data-dumps/redisgraph/`
  - `docker ps` -- get id
  - `docker stop {id}`
  - `docker rm {id}`
  - `docker volume ls` -- get id
  - `docker volume rm {id}`
  - `rm -rf db-container-data`

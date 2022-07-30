## Dev

I'll put these steps into makefile, however, I want to focus on my project first.

### Setup postgres with cargo data dump

- Download this: https://static.crates.io/db-dump.tar.gz
- Extract it into api/dumps/cargo/dump
- `docker build . -t custom-cargo-dump-db` from api/dumps/cargo/dump
- `cd ../../`
- `docker run -it --name deps-graph-postgres-cargo -d -p 7501:5432 -v $(pwd)/db-data/postgresCargo:/var/lib/postgresql/data -e POSTGRES_PASSWORD=c4rg0DUmP -e POSTGRES_USER=dumpuser -e POSTGRES_DB=dumpdb -e PGDATA=/var/lib/postgresql/data/pgdata custom-cargo-dump-db`
- `docker exec -it deps-graph-postgres-cargo bash`
- `cd /var/lib/postgresql/data`
- `psql dumpdb -U dumpuser < schema.sql`
- `psql dumpdb -U dumpuser < import.sql`

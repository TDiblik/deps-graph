FROM rust:1.61.0-alpine as builder
WORKDIR /code

ARG HOST_USER_NAME
ENV HOST=$HOST_USER_NAME
ENV UID=10001 
ENV CARGO_HOME=/code/.cargo

# Prepare builder image
RUN apk update \
 && apk add --no-cache musl-dev build-base libc-dev pkgconfig libressl-dev ca-certificates \
 && update-ca-certificates

# Create user with no privileges, other than running the binary. (https://stackoverflow.com/a/55757473/12429735 )
RUN adduser \    
    --disabled-password \    
    --gecos "" \    
    --home "/nonexistent" \    
    --shell "/sbin/nologin" \    
    --no-create-home \    
    --uid "${UID}" \    
    "${HOST}"

# Cach dependencies
RUN USER=root cargo new --bin api \
 && mv ./api/* . \
 && rm -r ./api/
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release  \
 && rm ./src/*.rs \
 && rm ./target/release/deps/api*

# Build the project
COPY ./src/ ./src/
RUN cargo build --release

# Host the project on totally blank distor
FROM scratch as host
WORKDIR /server

# Import the safe user and group files from the builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

ARG HOST_USER_NAME
ENV HOST=$HOST_USER_NAME
USER ${HOST}:${HOST}

# Copy the compiled binary and env files
COPY --from=builder --chown=${HOST}:${HOST} /code/target/release/api .
COPY --chown=${HOST}:${HOST} .env .

EXPOSE 8080
ENTRYPOINT [ "/server/api" ]
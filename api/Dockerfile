FROM rust:1.61.0-alpine as builder
WORKDIR /code

ARG HOST_USER_NAME
ENV HOST=$HOST_USER_NAME
ENV UID=10001 

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
RUN cargo new api --bin \
 && mv api/* . \
 && rm -r api/ \
 && rm Cargo.toml
COPY Cargo.toml Cargo.toml
RUN cargo build --release \
 && rm -rf src/ \
 && rm target/release/api

# Build the project
COPY src/ src/
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

# Copy the compiled binary
COPY --from=builder --chown=${HOST}:${HOST} /code/target/release/api .


EXPOSE 8080
ENTRYPOINT [ "/server/api" ]
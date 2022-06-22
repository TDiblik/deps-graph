# Build frontend
FROM node:16.15.0-alpine as nbuilder
WORKDIR /code

# Cache dependencies
COPY ./frontend/package.json package.json
COPY ./frontend/package-lock.json package-lock.json
RUN npm install --production

COPY ./frontend/ .
RUN npm run build --only=production

# Build api
FROM golang:1.18.3-alpine as builder
WORKDIR /code

# Prepare builder
RUN apk update \
 && apk add alpine-sdk git \
 && rm -rf /var/cache/apk/* \
 && go install github.com/swaggo/swag/cmd/swag@latest

# Cache dependencies
COPY ./api/go.mod .
COPY ./api/go.sum .
RUN go mod download

# Build the api
COPY ./api/ .
RUN rm -rf react-output \
 && swag init --parseDependency \
 && go build -o ./api -ldflags "-s -w"

# Create safe user;
FROM alpine as safeuser_builder

ARG HOST_USER_NAME
ENV HOST=$HOST_USER_NAME
ENV UID=10001 

RUN adduser \    
    --disabled-password \    
    --gecos "" \    
    --home "/nonexistent" \    
    --shell "/sbin/nologin" \    
    --no-create-home \    
    --uid "${UID}" \    
    "${HOST}"

# Host the project on totally blank distor
FROM alpine as host
WORKDIR /server

# Import the safe user and group files from the builder.
COPY --from=safeuser_builder /etc/passwd /etc/passwd
COPY --from=safeuser_builder /etc/group /etc/group

ARG HOST_USER_NAME
ENV HOST=$HOST_USER_NAME
USER ${HOST}:${HOST}

# Copy the compiled binary and env files
COPY --chown=${HOST}:${HOST} ./api/.env .
COPY --from=builder --chown=${HOST}:${HOST} /code/api .
COPY --from=nbuilder --chown=${HOST}:${HOST} /code/build/ ./react-output

EXPOSE 8080
ENTRYPOINT [ "/server/api" ]
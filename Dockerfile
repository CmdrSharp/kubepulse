################################
#### Build
FROM rust:1.69.0 as builder
ENV PKG_CONFIG_ALLOW_CROSS=1

# Build prep
RUN apt-get update
RUN apt-get install musl-tools libssl-dev build-essential -y
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src
RUN USER=root cargo new kubepulse
COPY Cargo.toml Cargo.lock /usr/src/kubepulse/
COPY templates /usr/src/kubepulse/templates
WORKDIR /usr/src/kubepulse
RUN cargo build --target x86_64-unknown-linux-musl --release
COPY src /usr/src/kubepulse/src/
RUN touch /usr/src/kubepulse/src/main.rs

# Actual build
RUN cargo build --target x86_64-unknown-linux-musl --release

################################
#### Runtime
FROM alpine:3.18 as runtime

WORKDIR /app

# Create the non-root user
RUN addgroup -S appadmin -g 1000 && adduser -S appadmin -G appadmin -D -u 1000

# Don't touch these
ENV LC_COLLATE en_US.UTF-8
ENV LC_CTYPE UTF-8
ENV LC_MESSAGES en_US.UTF-8
ENV LC_MONETARY en_US.UTF-8
ENV LC_NUMERIC en_US.UTF-8
ENV LC_TIME en_US.UTF-8
ENV LC_ALL en_US.UTF-8
ENV LANG en_US.UTF-8

# Copy the binary
COPY --from=builder /usr/src/kubepulse/target/x86_64-unknown-linux-musl/release/kubepulse /usr/local/bin/kubepulse
COPY --from=builder /usr/src/kubepulse/templates/assets /app/assets
RUN chmod +x /usr/local/bin/kubepulse
RUN chown appadmin:appadmin /usr/local/bin/kubepulse

# Run as non-root
USER appadmin
CMD ["/usr/local/bin/kubepulse"]

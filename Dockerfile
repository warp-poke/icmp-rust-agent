FROM rust:1.30.1-stretch

RUN apt-get update
RUN apt install pkg-config libssl-dev build-essential git libssl-dev libsasl2-dev libevent-pthreads-2.0-5 -y

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install

CMD ["myapp"]

# FROM alpine:edge as builder

# COPY src /source/src/
# COPY Cargo.toml /source/
# COPY Cargo.lock /source/
# #COPY config.toml /src/

# RUN apk add --no-cache --virtual .build-dependencies \
#     cargo \
#     build-base \
#     file \
#     libgcc \
#     libsasl \
#     musl-dev \
#     rust \
#     bash \
#     librdkafka-dev
# RUN apk add --no-cache 
# WORKDIR /source/
# RUN cargo build --release

# FROM alpine:edge
# COPY config.toml /etc/poke-agent/config.toml
# RUN apk add --no-cache openssl llvm-libunwind libgcc librdkafka
# COPY --from=builder /source/target/release/poke-agent /usr/bin/poke-agent
# CMD ["/usr/bin/poke-agent", "--help"]
FROM rust:1.85-bullseye AS base

WORKDIR /app
COPY ./Cargo.toml .
COPY ./src ./src
COPY ./rust-toolchain.toml .
RUN apt-get update
RUN apt-get install -y pkg-config openssl libssl-dev cmake gcc-mingw-w64-x86-64 nasm 
RUN export MINGW_PREFIX=/usr/x86_64-w64-mingw32/sys-root/mingw/
# RUN cargo build --target aarch64-unknown-linux-gnu --release
# RUN cargo build --target aarch64-pc-windows-msvc    --release
RUN cargo build --target x86_64-pc-windows-gnu     --release
RUN cargo build --target x86_64-unknown-linux-gnu  --release

FROM scratch AS export-stage
# COPY --from=base /app/target/aarch64-unknown-linux-gnu/release/picoman  ./picoman_linux_arm64
# COPY --from=base /app/target/aarch64-pc-windows-msvc/release/picoman.exe ./picoman_windows_arm64
COPY --from=base /app/target/x86_64-unknown-linux-gnu/release/picoman   ./picoman_linux_amd64
COPY --from=base /app/target/x86_64-pc-windows-msvc/release/picoman.exe  ./picoman_windows_amd64
PROFILE := "dev"

_dir_name := if PROFILE == "dev" { "debug" } else if PROFILE == "release" { "release" } else { "unknown" }

BIN_PATH := "target/" + _dir_name + "/picoman"
BIN_PATH_NIX := "target/" + _dir_name + "/picoman_nix"
BIN_PATH_WIN := "target/x86_64-pc-windows-gnu/release/picoman.exe"
BIN_PATH_LINUX_X86 := "target/x86_64-unknown-linux-gnu/release/picoman"
BIN_PATH_LINUX_ARM64 := "target/aarch64-unknown-linux-gnu/release/picoman"

WIN_TOOLCHAIN := "x86_64-pc-windows-gnu"
LINUX_X86_TOOLCHAIN := "x86_64-unknown-linux-gnu"
LINUX_ARM64_TOOLCHAIN := "aarch64-unknown-linux-gnu"

alias clear := clean
alias b := build
alias r := run
alias c := check

default: build

check:
	cargo check --profile {{PROFILE}}

build:
	cargo build --profile {{PROFILE}}

buildnix: #build and patch for nixos
	nix-shell --run "make build"
	cp {{BIN_PATH}} {{BIN_PATH_NIX}}
	nix-shell --run "patchelf --set-rpath $NIX_LD_LIBRARY_PATH {{BIN_PATH_NIX}}"

buildwin-x86:
	echo INSTALL mingw-w32 \& nasm before building!

	rustup target add {{WIN_TOOLCHAIN}}
	MINGW_PREFIX=/usr/x86_64-w64-mingw32/lib \
	cargo build --profile {{PROFILE}} --target {{WIN_TOOLCHAIN}}
#	cp target/x86_64-pc-windows-gnu

buildlinux-arm:
	cargo install cross --git https://github.com/cross-rs/cross

	~/.cargo/bin/cross build --profile {{PROFILE}} --target {{LINUX_ARM64_TOOLCHAIN}}

buildlinux-x86:
	# mkdir .temp-xargo
	# cargo install cross --git https://github.com/cross-rs/cross

	cargo build --profile {{PROFILE}} --target {{LINUX_X86_TOOLCHAIN}}

release: buildwin-x86 buildlinux-x86 # buildlinux-arm
	rm -rf build
	mkdir build
	cp LICENSE LICENSE.txt
	zip -9 build/picoman-{{WIN_TOOLCHAIN}}.zip {{BIN_PATH_WIN}} LICENSE.txt README.md
	zip -9 build/picoman-{{LINUX_X86_TOOLCHAIN}}.zip {{BIN_PATH_LINUX_X86}} LICENSE.txt README.md
	# zip -9 build/picoman-{{LINUX_ARM64_TOOLCHAIN}}-arm64.zip {{BIN_PATH_LINUX_ARM64}} LICENSE.txt README.md

	rm LICENSE.txt

	just check-size


check-size:
    du -sh {{BIN_PATH_LINUX_X86}} {{BIN_PATH_LINUX_ARM64}} {{BIN_PATH_WIN}} {{BIN_PATH}} build/*

run: build
	{{BIN_PATH}}

runnix: buildnix
	{{BIN_PATH_NIX}}

runwin:
	wine {{BIN_PATH_WIN}}


pull:
	git pull --recurse-submodules
	git submodule update --init --recursive --remote

clean:
	cargo clean
	rm -rf build

all: build

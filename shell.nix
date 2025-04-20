{ pkgs ? import <nixpkgs> {} }:
  let
    overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
    libPath = with pkgs; lib.makeLibraryPath [
      # load external libraries that you need in your rust project here
    ];
in 
pkgs.mkShell {
  buildInputs = [ 
    pkgs.cargo
    pkgs.rustc
    pkgs.rustup
#    pkgs.cargo-xbuild
    pkgs.cargo-xwin
    pkgs.cmake

    pkgs.openssl

  ];

  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

  RUSTC_VERSION = overrides.toolchain.channel;
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
  '';

  RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [

  ]);
  
  LD_LIBRARY_PATH = libPath;

  BINDGEN_EXTRA_CLANG_ARGS =
  # Includes normal include path
  (builtins.map (a: ''-I"${a}/include"'') [
    # add dev libraries here (e.g. pkgs.libvmi.dev)
    pkgs.glibc.dev
  ])
  # Includes with special directory paths
  ++ [
    ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
    ''-I"${pkgs.glib.dev}/include/glib-2.0"''
    ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
  ];

}
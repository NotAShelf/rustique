{pkgs ? import <nixpkgs> {}}: let
  inherit (pkgs.rustc) llvmPackages;
  runtimeDeps = with pkgs; [
    libxkbcommon
    wayland
    openssl
    sqlite
    glib
  ];
in
  pkgs.mkShell {
    name = "rustique";

    strictDeps = true;
    nativeBuildInputs = with pkgs; [
      pkg-config
      cargo
      rustc
      clang
      mold

      (rustfmt.override {asNightly = true;})
      rust-analyzer-unwrapped
      clippy
      taplo
    ];

    buildInputs = runtimeDeps;

    env = {
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
      LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath runtimeDeps}";
    };
  }

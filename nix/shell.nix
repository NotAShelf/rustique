{
  self,
  lib,
  stdenv,
  mkShell,
  clang,
  libclang,
  mold,
  pkg-config,
  taplo,
  libxkbcommon,
  vulkan-loader,
  wayland,
}: let
  rustiquePkg = self.packages.${stdenv.hostPlatform.system}.rustique;
  runtimeInputs = lib.makeLibraryPath [
    libxkbcommon
    vulkan-loader
    wayland
  ];
in
  mkShell {
    name = "rustique-dev";
    inputsFrom = [rustiquePkg];

    nativeBuildInputs = [
      clang
      mold
      pkg-config
      taplo
    ];
    env = {
      LIBCLANG_PATH = "${libclang.lib}/lib";
      LD_LIBRARY_PATH = "LD_LIBRARY_PATH:${runtimeInputs}/lib";
    };
  }

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
}: let
  rustiquePkg = self.packages.${stdenv.hostPlatform.system}.rustique;
  runtimeInputs = lib.makeLibraryPath rustiquePkg.buildInputs;
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
      LD_LIBRARY_PATH = "${runtimeInputs}:$LD_LIBRARY_PATH";
    };
  }

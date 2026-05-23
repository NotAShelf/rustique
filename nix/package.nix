{
  lib,
  craneLib,
  openssl,
  libxkbcommon,
  wayland,
}: let
  pname = "rustique";
  version = "0.6.0";

  buildInputs = [
    libxkbcommon
    wayland
    openssl
  ];

  commonArgs = {
    inherit pname version buildInputs;
    strictDeps = true;
    doCheck = false;

    src = let
      fs = lib.fileset;
      s = ../.;
    in
      fs.toSource {
        root = s;
        fileset = fs.intersection (fs.fromSource (craneLib.cleanCargoSource s)) (
          fs.unions [
            (s + /crates)
            (s + /packages)

            (s + /Cargo.toml)
            (s + /Cargo.lock)
          ]
        );
      };
  };

  # Pre-build all external deps, this derivation is cached across source changes
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      useNextest = true;

      meta = {
        description = "Fast, cross-platform mod manager for Vintage Story";
        homepage = "https://github.com/notashelf/rustique";
        license = lib.licenses.mit;
        maintainers = with lib.maintainers; [NotAShelf];
        platforms = lib.platforms.linux ++ lib.platforms.darwin;
        mainProgram = "rustique";
      };
    }
  )

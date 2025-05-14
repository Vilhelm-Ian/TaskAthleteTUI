# default.nix
{ pkgs ? import <nixpkgs> {}, task-athlete-lib }:

pkgs.rustPlatform.buildRustPackage {
  pname = "task-athlete-cli";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with pkgs.lib; {
    description = "A workout logger for the terminal";
    license = licenses.mit;
    maintainers = with maintainers; [ Vilhelm-Ian ];
  };
  buildInputs = [ task-athlete-lib ];
}


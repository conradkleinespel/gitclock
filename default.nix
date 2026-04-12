{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "gitclock";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ ];
  buildInputs = [ ];

  meta = with pkgs.lib; {
    description = "A CLI that helps protect your privacy when using git";
    homepage = "https://github.com/conradkleinespel/gitclock";
    license = licenses.asl20;
    maintainers = [ ];
  };
}

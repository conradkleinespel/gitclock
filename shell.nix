{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    cargo-tarpaulin
  ];

  shellHook = ''
    rustup default stable
    rustup component add rust-src
  '';
}

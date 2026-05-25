{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    podman
  ];

  shellHook = ''
    git config set core.hooksPath githooks

    rustup default stable
    rustup component add rust-src
  '';
}

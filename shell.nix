{ pkgs ? import <nixpkgs> {} }:

let
  mingw = pkgs.pkgsCross.mingwW64.buildPackages.gcc;
in
pkgs.mkShell {
  packages = with pkgs; [
    rustc
    cargo
    rustfmt
    gcc
    binutils
    gdb
    mingw
  ];
}

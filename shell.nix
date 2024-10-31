{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.llvm_18
  ];

  shellHook = ''
    export LLVM_SYS_180_PREFIX=$(dirname $(dirname $(which llvm-config)))
  '';
}


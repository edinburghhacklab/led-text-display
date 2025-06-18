{
  pkgs ? import <nixpkgs> { },
  pkgsCross ? import <nixpkgs> {
    crossSystem = {
      config = "armv7l-unknown-linux-gnueabihf";
    };
  },
}:
pkgs.mkShell {
  nativeBuildInputs = [
    pkgsCross.buildPackages.gcc
    pkgs.SDL2
  ];
}

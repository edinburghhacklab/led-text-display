{
  pkgs ? import <nixpkgs> {
    crossSystem = {
      config = "armv7l-unknown-linux-gnueabihf";
    };
  },
}:
pkgs.mkShell {
  nativeBuildInputs = [ pkgs.buildPackages.gcc ];
}

{
  inputs = {
    nixpkgs.url      = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let 
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-analyzer
            (rust-bin.nightly."2022-07-10".default.override {
              extensions = [
                "rust-src"
                "cargo"
                "rustc"
                "rustfmt"
                "miri"
              ];
            })
            cargo-expand
            cargo-readme
            cargo-msrv
          ];
        };
      }
    );
}

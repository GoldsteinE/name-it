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
          RUST_LOG = "info";
          buildInputs = with pkgs; [
            (rust-bin.nightly."2022-08-09".default.override {
              extensions = [
                "rust-src"
                "cargo"
                "rustc"
                "rustfmt"
                "miri"
                "rust-analyzer"
              ];
            })
            cargo-expand
            cargo-readme
            cargo-msrv
            act
            actionlint
            # for tools
            openssl
            pkg-config
          ];
        };
      }
    );
}

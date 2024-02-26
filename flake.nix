
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        fenixChannel = fenix.packages.${system}.stable;
        fenixToolchain = (fenixChannel.withComponents [
          "rustc"
          "cargo"
          "rustfmt"
          "clippy"
          "rust-analysis"
          "rust-src"
          "llvm-tools-preview"
        ]);
        craneLib = crane.lib.${system}.overrideToolchain fenixToolchain;

        # Used in both dev shell and to build the app in nix eventually
        commonArgs = {
          nativeBuildInputs = with pkgs; [ 
		  fenixToolchain 
		  valgrind
		  # pkg-config 
	  ];

          buildInputs = with pkgs; [
		cargo-audit
		cargo-auditable
		tokio-console
            # glib
            # openssl
          ];
        };
      in
    {
      devShells.default = pkgs.mkShell commonArgs // {
        RUST_SRC_PATH = "${fenixChannel.rust-src}/lib/rustlib/src/rust/library";
      };

      packages.default = craneLib.buildPackage ( commonArgs // {
        src = craneLib.cleanCargoSource ./.;
      });
    });
}

{
  description = "A Discord bot tracking the delay of certain tardy friends joining a Discord call.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        systemsToBuildFor = ["x86_64-linux" "aarch64-linux"];
        # Attribute set of all targets to compile for.
        # This will be referenced in outputs when we define what target we want.
        # targetPkgs = lib.attrsets.genAttrs systemsToBuildFor (target:
        #   import nixpkgs {
        #     crossSystem = target;
        #     inherit localSystem;
        #     overlays = [ (import rust-overlay ) ];
        #   };
        # );
        pkgs = import nixpkgs {
          inherit localSystem;
          crossSystem = "x86_64-linux";
          overlays = [ (import rust-overlay ) ];
        };

        # Pin Rust toolchain via overlay.
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Setup Crane functions with pinned toolchain in project.
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Separate out the migrations to use later
        # for the SQLx CLI setup.
        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            ./migrations
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        mst-bot-crate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          nativeBuildInputs = (commonArgs.nativeBuildInputs ++ [
            pkgs.sqlx-cli
          ]);
          pre-build = ''
            export DATABASE_URL=sqlite:./database.db
            sqlx database create
            sqlx migrate run
          '';
        });


        crateExpression =
        {
            openssl,
            darwin,
            libiconv,
            lib,
            pkg-config,
            stdenv,
        }:
        let
          commonArgs = {
            inherit src;
            strictDeps = true;

            nativeBuildInputs = [
              pkg-config
            ];

            buildInputs = [
              openssl
            ] ++ lib.optionals pkgs.stdenv.isDarwin [
              # Hacky macOS dependencies. Wonderful.
              libiconv
              darwin.apple_sdk.frameworks.Security
            ];
          };
        in
        craneLib.buildPackage {
          inherit src;
          strictDeps = true;
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          nativeBuildInputs = [ pkg-config ] ++ lib.optionals stdenv.buildPlatform.isDarwin [ libiconv ];
          buildInputs = [ openssl ];
        };
        
      in 
    )
}

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
  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      localSystem: let
        systemsToBuildFor = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [(import rust-overlay)];
        };
        # Pin Rust toolchain via overlay.
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        inherit (pkgs) lib;

        # Setup Crane functions with pinned toolchain in project.
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Function to generate the relevant Crane package for the system target selected.
        #
        # This lets us eventually generate one crane package for each cross-compilation build
        # we're doing.
        generateCranePackage = (
          crossSystem: let
            pkgs = import nixpkgs {
              inherit localSystem crossSystem;
              overlays = [(import rust-overlay)];
            };
            inherit (pkgs) lib;

            # Setup Crane functions with pinned toolchain in project.
            craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);

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

            commonArgs = {
              inherit src;
              strictDeps = true;
              nativeBuildInputs = with pkgs.pkgsBuildHost; [
                pkg-config
              ];
              buildInputs = with pkgs.pkgsHostHost;
                [
                  openssl
                ]
                ++ lib.optionals stdenv.isDarwin [
                  libiconv
                  darwin.apple_sdk.frameworks.Security
                ];
            };
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          in
            craneLib.buildPackage (commonArgs
              // {
                inherit cargoArtifacts;
                nativeBuildInputs =
                  (commonArgs.nativeBuildInputs or [])
                  ++ [
                    pkgs.pkgsBuildHost.sqlx-cli
                  ];
                preBuild = ''
                  export DATABASE_URL=sqlite:./database.db
                  sqlx database create
                  sqlx migrate run
                '';
              })
        );
        # Filter the available cross-compilation system for x86_64-linux and aarch64-linux
        # to exclude macOS packages. Unfortunately, certain libraries are unavailable when
        # cross-compiling for macOS.
        availableSystems =
          if localSystem != "aarch64-darwin"
          then builtins.filter (x: x != "aarch64-darwin") systemsToBuildFor
          else systemsToBuildFor;
        cranePackages = lib.attrsets.genAttrs availableSystems (system: generateCranePackage system);
        dockerImagesForSystem =
          map (crossSystem: let
            pkgs = import nixpkgs {
              inherit localSystem crossSystem;
              overlays = [(import rust-overlay)];
            };
          in {
            name = "docker-${crossSystem}";
            value = pkgs.pkgsHostHost.dockerTools.streamLayeredImage {
              name = "mst-bot";
              tag = "latest";
              contents = with pkgs.pkgsHostHost; [
                cranePackages."${crossSystem}"
                cacert
                (pkgs.buildEnv
                {
                  name = "mst-bot-migrations";
                  paths = [./migrations];
                })
              ];
              config = {
                Cmd = ["${cranePackages."${crossSystem}"}/bin/mst-bot"];
              };
            };
          })
          availableSystems;
        dockerImages = builtins.listToAttrs dockerImagesForSystem;
      in {
        checks =
          cranePackages
          // {
            default = cranePackages."${localSystem}";
          };

        packages =
          cranePackages
          // dockerImages
          // {
            default = cranePackages."${localSystem}";
          };

        devShells.default = craneLib.devShell {
          checks = self.checks;
          packages = with pkgs; [
            sqlx-cli
            ripgrep
            fd
          ];
        };
      }
    );
}

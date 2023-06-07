{
  description = "ACVM Simulator";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-22.11";
    };

    flake-utils = {
      url = "github:numtide/flake-utils";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      # All of these inputs (a.k.a. dependencies) need to align with inputs we
      # use so they use the `inputs.*.follows` syntax to reference our inputs
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    crane = {
      url = "github:ipetkov/crane";
      # All of these inputs (a.k.a. dependencies) need to align with inputs we
      # use so they use the `inputs.*.follows` syntax to reference our inputs
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        flake-compat.follows = "flake-compat";
        rust-overlay.follows = "rust-overlay";
      };
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, rust-overlay, ... }: #,
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };

      rustToolchain = pkgs.rust-bin.stable."1.66.0".default.override {
        # We include rust-src to ensure rust-analyzer works.
        # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/4
        extensions = [ "rust-src" ];
        targets = [ "wasm32-unknown-unknown" ]
          ++ pkgs.lib.optional (pkgs.hostPlatform.isx86_64 && pkgs.hostPlatform.isLinux) "x86_64-unknown-linux-gnu"
          ++ pkgs.lib.optional (pkgs.hostPlatform.isAarch64 && pkgs.hostPlatform.isLinux) "aarch64-unknown-linux-gnu"
          ++ pkgs.lib.optional (pkgs.hostPlatform.isx86_64 && pkgs.hostPlatform.isDarwin) "x86_64-apple-darwin"
          ++ pkgs.lib.optional (pkgs.hostPlatform.isAarch64 && pkgs.hostPlatform.isDarwin) "aarch64-apple-darwin";
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      sharedEnvironment = {};

      wasmEnvironment = sharedEnvironment // {};

      sourceFilter = path: type:
        (craneLib.filterCargoSources path type);

      # The `self.rev` property is only available when the working tree is not dirty
      GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
      GIT_DIRTY = if (self ? rev) then "false" else "true";

      commonArgs = {
        pname = "acvm-simulator";
        version = "0.0.0"; # x-release-please-version

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = sourceFilter;
        };

        cargoClippyExtraArgs = "--all-targets -- -D warnings";
        # cargoTestExtraArgs = "--workspace";

        # We don't want to run checks or tests when just building the project
        doCheck = false;
      };

      # Combine the environment and other configuration needed for crane to build with the wasm feature
      wasmArgs = wasmEnvironment // commonArgs // {

        cargoExtraArgs = "--target=wasm32-unknown-unknown";

        buildInputs = [ ];

      };

      # Build *just* the cargo dependencies, so we can reuse all of that work between runs
      cargoArtifacts = craneLib.buildDepsOnly wasmArgs;

      wasm-bindgen-cli = pkgs.callPackage ./nix/wasm-bindgen-cli/default.nix {
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustToolchain;
          cargo = rustToolchain;
        };
      };
    in
    rec {
      checks = {
        cargo-clippy = craneLib.cargoClippy (wasmArgs // {
          inherit cargoArtifacts;
          inherit GIT_COMMIT GIT_DIRTY;

          doCheck = true;
        });
      };

      packages.default = craneLib.mkCargoDerivation (wasmArgs // rec {
        inherit cargoArtifacts;
        inherit GIT_COMMIT;
        inherit GIT_DIRTY;

        COMMIT_SHORT = builtins.substring 0 7 GIT_COMMIT;
        VERSION_APPENDIX = if GIT_DIRTY == "true" then "-dirty" else "";

        src = ./.; #craneLib.cleanCargoSource (craneLib.path ./.);

        nativeBuildInputs = with pkgs; [
          binaryen
          which
          git
          jq
          rustToolchain
          wasm-bindgen-cli
        ];

        buildPhaseCargoCommand = ''
          bash ./buildPhaseCargoCommand.sh
        '';

        installPhase = ''
          bash ./installPhase.sh        
        '';

      });

      # Setup the environment to match the stdenv from `nix build` & `nix flake check`, and
      # combine it with the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell (wasmEnvironment // {
        # inputsFrom = builtins.attrValues checks;

        nativeBuildInputs = with pkgs; [
          starship
          nil
          nixpkgs-fmt
          which
          git
          jq
          rustToolchain
          wasm-bindgen-cli
          nodejs
          yarn
        ];

        shellHook = ''
          eval "$(starship init bash)"
        '';
      });
    });
}
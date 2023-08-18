{
  description = "Javascript bindings for the ACVM";

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
    barretenberg = {
      url = "github:AztecProtocol/barretenberg";
      # All of these inputs (a.k.a. dependencies) need to align with inputs we
      # use so they use the `inputs.*.follows` syntax to reference our inputs
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, rust-overlay, barretenberg, ... }: #, barretenberg
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
          barretenberg.overlays.default
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

      crateACVMJSDefinitions = craneLib.crateNameFromCargoToml {
        cargoToml = ./acvm_js/Cargo.toml;
      };

      crateACVMDefinitions = craneLib.crateNameFromCargoToml {
        cargoToml = ./acvm/Cargo.toml;
      };


      sharedEnvironment = {
        # Barretenberg fails if tests are run on multiple threads, so we set the test thread
        # count to 1 throughout the entire project
        #
        # Note: Setting this allows for consistent behavior across build and shells, but is mostly
        # hidden from the developer - i.e. when they see the command being run via `nix flake check`
        # RUST_TEST_THREADS = "1";
      };

      nativeEnvironment = sharedEnvironment // {
        # rust-bindgen needs to know the location of libclang
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };

      wasmEnvironment = sharedEnvironment // {
        BARRETENBERG_BIN_DIR = "${pkgs.barretenberg-wasm}/bin";
      };

      sourceFilter = path: type:
        (craneLib.filterCargoSources path type);

      # As per https://discourse.nixos.org/t/gcc11stdenv-and-clang/17734/7 since it seems that aarch64-linux uses
      # gcc9 instead of gcc11 for the C++ stdlib, while all other targets we support provide the correct libstdc++
      stdenv =
        if (pkgs.stdenv.targetPlatform.isGnu && pkgs.stdenv.targetPlatform.isAarch64) then
          pkgs.overrideCC pkgs.llvmPackages.stdenv (pkgs.llvmPackages.clang.override { gccForLibs = pkgs.gcc11.cc; })
        else
          pkgs.llvmPackages.stdenv;

      extraBuildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
        # Need libiconv and apple Security on Darwin. See https://github.com/ipetkov/crane/issues/156
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.Security
      ];

      # The `self.rev` property is only available when the working tree is not dirty
      GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
      GIT_DIRTY = if (self ? rev) then "false" else "true";

      commonArgs = {
        inherit (crateACVMDefinitions) pname version;
        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path {
            path = ./.;
          };
          filter = sourceFilter;
        };

        cargoTestExtraArgs = "--workspace";

        # We don't want to run checks or tests when just building the project
        doCheck = false;
      };

      # Combine the environment and other configuration needed for crane to build with the native feature
      nativeArgs = nativeEnvironment // commonArgs // {
        # Use our custom stdenv to build and test our Rust project
        inherit stdenv;

        nativeBuildInputs = [
          # This provides the pkg-config tool to find barretenberg & other native libraries
          pkgs.pkg-config
          # This provides the `lld` linker to cargo
          pkgs.llvmPackages.bintools
        ];

        buildInputs = [
          pkgs.llvmPackages.openmp
          pkgs.barretenberg
        ] ++ extraBuildInputs;
      };

      # Combine the environment and other configuration needed for crane to build with the native feature
      wasmArgs = wasmEnvironment // commonArgs // {
        # Use our custom stdenv to build and test our Rust project
        inherit stdenv;

        cargoExtraArgs = "--no-default-features --features='bn254, wasm'";
      };

      # Combine the environment and other configuration needed for crane to build with the wasm feature
      acvmjsWasmArgs = wasmEnvironment // commonArgs // {

        inherit (crateACVMJSDefinitions) pname version;

        cargoExtraArgs = "--package acvm_js --target=wasm32-unknown-unknown --no-default-features --features='bn254'";

        cargoVendorDir = craneLib.vendorCargoDeps {
          src = ./.;
        };

        buildInputs = [ ];

      };

      ## ACVM Rust Library

      # Build *just* the cargo dependencies, so we can reuse all of that work between runs
      acvm-native-cargo-artifacts = craneLib.buildDepsOnly nativeArgs;
      acvm-wasm-cargo-artifacts = craneLib.buildDepsOnly commonArgs;
      acvm-js-cargo-artifacts = craneLib.buildDepsOnly acvmjsWasmArgs;

      acvm-native = craneLib.buildPackage (nativeArgs // {
        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = acvm-native-cargo-artifacts;
      });

      acvm-wasm = craneLib.buildPackage (wasmArgs // {
        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = acvm-wasm-cargo-artifacts;
      });

      ## ACVM JS stuff

      wasm-bindgen-cli = pkgs.callPackage ./acvm_js/nix/wasm-bindgen-cli/default.nix {
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustToolchain;
          cargo = rustToolchain;
        };
      };

      acvm-js = craneLib.buildPackage (acvmjsWasmArgs // {
        cargoArtifacts = acvm-js-cargo-artifacts;

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

        CARGO_TARGET_DIR = "./target";

        buildPhaseCargoCommand = ''
          bash ./acvm_js/buildPhaseCargoCommand.sh
        '';

        installPhase = ''
          bash ./acvm_js/installPhase.sh        
        '';
      });

    in
    rec {
      checks = {

        cargo-clippy = craneLib.cargoClippy (commonArgs // sharedEnvironment // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = acvm-native-cargo-artifacts;
          doCheck = true;
        });

        cargo-test = craneLib.cargoTest (commonArgs // sharedEnvironment // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = acvm-native-cargo-artifacts;
          doCheck = true;
        });

        cargo-fmt = craneLib.cargoFmt (commonArgs // sharedEnvironment // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = acvm-native-cargo-artifacts;
          doCheck = true;
        });

      };

      packages = {
        inherit acvm-native;
        inherit acvm-wasm;
        inherit acvm-js;

        default = acvm-native;
      };

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
          toml2json
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

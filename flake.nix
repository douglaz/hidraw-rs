{
  description = "Rust HID library for Linux hidraw interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ "x86_64-unknown-linux-musl" ];
        };
      in
      {
        # Default package: hidraw-rs library
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "hidraw-rs";
          version = "0.1.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];
          
          # Minimal dependencies - only libc for system calls
          buildInputs = [ ];
          
          meta = with pkgs.lib; {
            description = "Rust HID library for Linux hidraw interface";
            homepage = "https://github.com/yourusername/hidraw-rs";
            license = with licenses; [ mit asl20 ];
            maintainers = [ ];
          };
        };
        
        # Static musl build
        packages.hidraw-rs-static = let
          rustPlatformMusl = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
        in rustPlatformMusl.buildRustPackage {
          pname = "hidraw-rs-static";
          version = "0.1.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
            pkgsStatic.stdenv.cc
          ];
          
          buildInputs = [ ];
          
          # Force cargo to use the musl target
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
          CC_x86_64_unknown_linux_musl = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C link-arg=-static";
          
          doCheck = false; # Tests don't work well with static linking
          
          meta = with pkgs.lib; {
            description = "Rust HID library for Linux hidraw interface (static build)";
            homepage = "https://github.com/yourusername/hidraw-rs";
            license = with licenses; [ mit asl20 ];
            maintainers = [ ];
          };
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bashInteractive
            rustToolchain
            pkg-config
            
            # Development tools
            cargo-watch
            cargo-expand
            rust-analyzer
            
            # For running examples and testing
            usbutils # lsusb command
          ];
          
          shellHook = ''
            # Set up Git hooks if not already configured
            if [ -d .git ] && [ -d .githooks ]; then
              current_hooks_path=$(git config core.hooksPath || echo "")
              if [ "$current_hooks_path" != ".githooks" ]; then
                echo "📎 Setting up Git hooks for code quality checks..."
                git config core.hooksPath .githooks
                echo "✅ Git hooks configured automatically!"
                echo "   • pre-commit: Checks code formatting"
                echo "   • pre-push: Runs formatting and clippy checks"
                echo ""
                echo "To disable: git config --unset core.hooksPath"
                echo ""
              fi
            fi
            
            echo "hidraw-rs development environment"
            echo ""
            echo "Available commands:"
            echo "  cargo build              - Build the library"
            echo "  cargo test               - Run tests"
            echo "  cargo run --example list_devices - List HID devices"
            echo "  cargo run --example basic_hid    - Basic HID example"
            echo "  cargo run --example coldcard_ping - Coldcard ping example"
            echo ""
            echo "Note: Some examples may require root permissions to access HID devices"
          '';
        };
      }
    );
}
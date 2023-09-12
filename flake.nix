{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, flake-parts, rust-overlay, crane, nixpkgs, ... }: 
    flake-parts.lib.mkFlake {inherit inputs;} {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem = {pkgs, system, ... }: 
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          commonArgs = {
            src = craneLib.cleanCargoSource (craneLib.path ./.);

            nativeBuildInputs = with pkgs; [
              pkg-config
              openssl
            ];
                       
            buildInputs = with pkgs; [    
              udev
              libxkbcommon
              vulkan-loader

              wayland             
              libGL

              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
              xorg.libxcb
              xorg.libX11
            ];

          };
                    
          craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.stable.latest.default;

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          crustility = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            postFixup = with pkgs; ''
              patchelf --set-rpath "${
                pkgs.lib.makeLibraryPath (commonArgs.buildInputs)
              }" \
              $out/bin/crustility
            '';
            
          });
        in {

          packages = {
            inherit crustility;
            default = crustility;
          };

          devShells.default = pkgs.mkShell
            {
              
              buildInputs = commonArgs.buildInputs;
              nativeBuildInputs = with pkgs; [
                (rust-bin.stable.latest.default.override { extensions = [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rust-analyzer"
                  "rustc"
                  "rustfmt"
                ];})
              ] ++ commonArgs.nativeBuildInputs;

              LD_LIBRARY_PATH=pkgs.lib.makeLibraryPath (commonArgs.buildInputs);
            };
        };
    };
}

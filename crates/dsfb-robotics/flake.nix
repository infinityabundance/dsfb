{
  description = "dsfb-robotics — pinned reviewer reproduction environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable."1.85.1".default.override {
          extensions = [ "rust-src" "rust-docs" ];
          targets = [
            "thumbv7em-none-eabihf"
            "riscv32imac-unknown-none-elf"
            "x86_64-unknown-linux-gnu"
          ];
        };
        python = pkgs.python312.withPackages (ps: with ps; [
          numpy
          matplotlib
          h5py
          pyarrow
          plotly
          scipy
        ]);
        texlive = pkgs.texlive.combine {
          inherit (pkgs.texlive) scheme-medium
            booktabs array longtable enumitem
            hyperref xcolor fancyhdr titlesec tocloft
            float listings tcolorbox tabularx
            caption subcaption multicol;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "dsfb-robotics-dev";
          buildInputs = [
            rust
            python
            texlive
            pkgs.cmake
            pkgs.eigen
            pkgs.gcc
            pkgs.gnumake
            pkgs.pkg-config
            pkgs.gh
            pkgs.git
            pkgs.valgrind
            pkgs.jq
          ];
          shellHook = ''
            echo "dsfb-robotics reviewer environment"
            echo "  rustc: $(rustc --version)"
            echo "  python: $(python3 --version)"
            echo "  pdflatex: $(pdflatex --version | head -1)"
            echo "  cmake: $(cmake --version | head -1)"
            echo "  eigen: $(pkg-config --modversion eigen3)"
            echo
            echo "Reproduce the paper:"
            echo "  bash scripts/reproduce.sh"
            echo "or step by step:"
            echo "  bash scripts/build_panda_gaz_model.sh"
            echo "  python3 scripts/preprocess_datasets.py"
            echo "  python3 scripts/compute_published_residuals.py"
            echo "  cargo build --release --features std,paper_lock --bin paper-lock"
            echo "  python3 scripts/bootstrap_census.py"
            echo "  python3 scripts/sensitivity_grid.py"
            echo "  python3 scripts/ablation.py"
            echo "  cd paper && latexmk -pdf dsfb_robotics.tex"
          '';
        };

        # Quick build target: paper-lock binary only.
        packages.paper-lock = pkgs.rustPlatform.buildRustPackage {
          pname = "dsfb-robotics-paper-lock";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--features" "std,paper_lock" "--bin" "paper-lock" ];
          doCheck = false;
        };

        packages.default = self.packages.${system}.paper-lock;
      });
}

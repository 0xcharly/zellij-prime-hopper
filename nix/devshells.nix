{
  perSystem = {
    config,
    lib,
    pkgs,
    ...
  }: {
    devShells.default = pkgs.mkShell {
      buildInputs =
        [
          # Support tools.
          pkgs.just # Command runner

          # Nix tools.
          pkgs.nixd # LSP
          pkgs.alejandra # Formatter

          # Markdown tools.
          pkgs.markdownlint-cli # LSP

          # Rust tools.
          pkgs.bacon # Diagnostics
          pkgs.rust-analyzer # LSP
          (pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml) # Toolchain
        ]
        ++ lib.optionals pkgs.stdenv.isLinux [
          # Dependencies needed for bacon to build on Linux.
          pkgs.openssl
          pkgs.pkg-config
        ];

      formatter = config.treefmt.build.wrapper;

      # Set up pre-commit hooks when user enters the shell.
      shellHook = let
        inherit (pkgs) lib;
        recipes = {
          fmt = {
            text = ''${lib.getExe config.treefmt.build.wrapper} --on-unmatched=info'';
            doc = "Format all files in this directory and its subdirectories.";
          };
        };
        commonJustfile = pkgs.writeTextFile {
          name = "justfile.incl";
          text =
            lib.concatStringsSep "\n"
            (lib.mapAttrsToList (name: recipe: ''
                [doc("${recipe.doc}")]
                ${name}:
                    ${recipe.text}
              '')
              recipes);
        };
      in ''
        ${config.pre-commit.installationScript}
        ln -sf ${builtins.toString commonJustfile} ./.justfile.incl
      '';
    };
  };
}

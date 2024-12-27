{inputs, ...}: {
  imports = [inputs.treefmt-nix.flakeModule];

  perSystem = {
    treefmt = {
      projectRootFile = "flake.lock";

      programs = {
        alejandra.enable = true;
        just.enable = true;
        mdformat.enable = true;
        rustfmt.enable = true;
        stylua.enable = true;
      };

      settings.formatter = {
        just.includes = ["*/.justfile"];
      };
      settings.global.excludes = [".envrc" "LICENSE" "*.kdl" "*.toml"];
    };
  };
}

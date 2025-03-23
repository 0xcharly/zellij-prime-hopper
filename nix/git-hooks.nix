{inputs, ...}: {
  imports = [inputs.git-hooks-nix.flakeModule];

  perSystem = {
    lib,
    pkgs,
    ...
  }: {
    pre-commit = {
      inherit pkgs;
      settings = {
        hooks = {
          alejandra.enable = true;
          autoflake.enable = true;
          check-builtin-literals.enable = true;
          check-case-conflicts.enable = true;
          end-of-file-fixer.enable = true;
          flake-checker.enable = true;
          just = {
            enable = true;
            name = "just";
            description = "Format just files";
            files = ".justfile$";
            entry = "bash -c 'for f in \"$@\"; do ${lib.getExe pkgs.just} --fmt --unstable --justfile $f; done'";
          };
          markdownlint = {
            enable = true;
            settings.configuration = {
              MD034 = false;
              MD040 = false; # fenced-code-language
              MD013 = false; # line-length
            };
          };
          ripsecrets.enable = true;
          rustfmt.enable = true;
          trim-trailing-whitespace.enable = true;
        };
      };
    };
  };
}

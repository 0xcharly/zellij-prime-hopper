{inputs, ...}: {
  imports = [inputs.flake-parts.flakeModules.easyOverlay];

  perSystem = {
    config,
    pkgs,
    ...
  }: {
    overlayAttrs = {
      default = config.packages;
      zellij-prime-hopper = config.packages.default;
    };
    packages.default = pkgs.callPackage ./mk-zellij-prime-hopper.nix {};
  };
}

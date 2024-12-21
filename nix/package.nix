{inputs, ...}: {
  imports = [inputs.flake-parts.flakeModules.easyOverlay];

  perSystem = {
    config,
    pkgs,
    ...
  }: {
    overlayAttrs.default = config.packages;
    packages.default = pkgs.callPackage ./mk-zellij-prime-hopper.nix {};
  };
}

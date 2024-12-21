{inputs, ...}: {
  perSystem = {system, ...}: let
    isDarwin = system == "aarch64-darwin";
    nixpkgs =
      if isDarwin
      then inputs.nixpkgs-darwin
      else inputs.nixpkgs;
  in {
    _module.args.pkgs = import nixpkgs {
      inherit system;
      overlays = [inputs.rust-overlay.overlays.default];
    };
  };
}

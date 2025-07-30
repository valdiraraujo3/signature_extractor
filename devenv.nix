{
  pkgs,
  ...
}:
{
  imports = [
    {
      packages = [
        pkgs.openssl
        pkgs.tesseract
        pkgs.graalvmPackages.graalvm-ce
      ];
    }
  ];
  config = {
    languages.rust = {
      channel = "stable";
      enable = true;
    };
    git-hooks.hooks = {
      rustfmt = {
        enable = true;
        files = "\.rs$";
      };
      clippy = {
        enable = true;
        settings = {
          denyWarnings = true;
          allFeatures = true;
        };
      };
    };

  };
}

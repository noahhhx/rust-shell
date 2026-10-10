{
  pkgs,
  ...
}:

{
  # https://devenv.sh/packages/
  packages = [ pkgs.git ];

  # https://devenv.sh/languages/rust/
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  # https://devenv.sh/tests/
  enterTest = ''
    cargo clippy --version
    cargo fmt --version
  '';
}

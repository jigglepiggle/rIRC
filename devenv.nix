
{ pkgs, ... }:

{
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  packages = with pkgs; [
    ngircd
    openssl
    pkg-config
  ];

  enterShell = ''
    echo "IRC client development environment"
    echo "Rust: $(rustc --version)"
    echo "Server: $(ngircd --version 2>&1 | head -n 1)"
  '';
}


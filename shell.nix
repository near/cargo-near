with import <nixpkgs> {};
stdenv.mkDerivation rec {
    name = "TestShell";
    src = null;
    buildInputs = [ rustup openssl pkg-config systemd ];
}

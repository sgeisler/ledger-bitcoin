with import <nixpkgs> {};

stdenv.mkDerivation rec {
    name = "rust";
    buildInputs = [
        libudev
        pkg-config
    ];

    LD_LIBRARY_PATH=lib.makeLibraryPath buildInputs;
}

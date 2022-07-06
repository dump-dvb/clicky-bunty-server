{ naersk, src, lib, pkg-config, cmake, protobuf, stops, zlib }:

naersk.buildPackage {
  pname = "clicky-bunty-backend";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ pkg-config ];

  meta = with lib; {
    description = "Backend for users to configure and register their stations";
    homepage = "https://github.com/dump-dvb/clicky-bunty-server";
  };
}

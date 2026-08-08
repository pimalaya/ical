# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  buildFeatures ? [ ],
  buildNoDefaultFeatures ? false,
  buildPackages,
  fetchFromGitHub,
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  inherit buildNoDefaultFeatures buildFeatures;

  pname = "ical-rs";
  version = "0.0.1";
  cargoHash = "";

  src = fetchFromGitHub {
    owner = "pimalaya";
    repo = "ical";
    rev = "v0.0.1";
    hash = "";
  };

  meta = {
    description = "iCalendar (RFC 5545) model with parser and builder, written in Rust";
    homepage = "https://github.com/pimalaya/ical";
    changelog = "https://github.com/pimalaya/ical/blob/master/CHANGELOG.md";
    license = [
      lib.licenses.mit
      lib.licenses.asl20
    ];
    maintainers = with lib.maintainers; [ soywod ];
  };
}

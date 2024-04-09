{
	description = "winvoice-server dev env / packaging";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = inputs @ { self, nixpkgs, ... }:
	let
		inherit (nixpkgs) lib;
		inherit (self) outputs;
		genSystems = lib.genAttrs ["aarch64-darwin" "aarch64-linux" "i686-linux" "x86_64-darwin" "x86_64-linux"];

		mkPkgs = system: import nixpkgs {
			inherit system;
			config.allowUnfree = true;
			overlays = (builtins.attrValues outputs.overlays) ++ [
				inputs.fenix.overlays.default
			];
		};
	in {
		devShells = genSystems (system:
		let pkgs = mkPkgs system;
		in {
			default = pkgs.mkShell {
				name = "winvoice-server";

				inputsFrom = with pkgs; [
					openssl
				];

				packages = with pkgs; [
					# build
					(fenix.fromToolchainFile {
						file = ./rust-toolchain.toml;
						sha256 = "sha256-7QfkHty6hSrgNM0fspycYkRcB82eEqYa4CoAJ9qA3tU=";
					})

					# deploy
					kind
					kubectl
					kubectl-cnpg
				];
			};
		});

		overlays = { };
	};
}

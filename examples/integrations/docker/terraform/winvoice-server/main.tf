terraform {
	required_providers {
		docker = {
			source = "kreuzwerker/docker"
			version = "~> 3.0"
		}
	}
}

# postgres

resource "docker_image" "winvoice-server" {
	name = "winvoice-server:0.6.3"
	build {
		context = "."
		dockerfile = "../../Dockerfile"
		build_arg = var.build-args
		platform = var.target-platform
	}
}

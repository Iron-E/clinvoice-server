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
	name = "winvoice-server:0.6.2"
	build {
		context = "."
		dockerfile = "../../Dockerfile"
		build_arg = var.build-args
		platform = var.target-platform
	}
}

resource "docker_container" "winvoice-server" {
	name = "winvoice/server"
	image = docker_image.winvoice-server.image_id
}

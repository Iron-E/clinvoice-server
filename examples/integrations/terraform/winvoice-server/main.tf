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
	name = "postgres:16.2"
}

resource "docker_container" "winvoice-server" {
	name = "winvoice-db-postgres"
	image = docker_image.winvoice-server
}

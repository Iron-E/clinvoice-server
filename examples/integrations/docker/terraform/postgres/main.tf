terraform {
	required_providers {
		docker = {
			source = "kreuzwerker/docker"
			version = "~> 3.0"
		}
	}
}

# postgres

resource "docker_image" "postgres" {
	name = "postgres:16.2"
}

resource "docker_container" "winvoice-db-postgres" {
	name = "winvoice/db/postgres"
	image = docker_image.postgres.image_id
}

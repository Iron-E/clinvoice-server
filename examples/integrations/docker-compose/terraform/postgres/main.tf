terraform {
	required_providers {
		docker = {
			source = "kreuzwerker/docker"
			version = "~> 3.0"
		}
	}
}

module "postgres" {
	source = "../../../docker/terraform/postgres"
}

# postgres

resource "docker_service" "db" {
	name = "db"

	task_spec {
		container_spec {
			image = module.postgres.image
		}
	}
}

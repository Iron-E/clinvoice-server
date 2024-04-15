terraform {
	required_providers {
		docker = {
			source  = "kreuzwerker/docker"
			version = "~> 3.0"
		}
	}
}

module "postgres" {
	source = "../../docker/terraform/postgres"
}

module "winvoice-server" {
	source = "../../docker/terraform/postgres"
}

resource "docker_service" "db" {
   field = "value"
}

terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
}

provider "docker" { }

# postgres

resource "docker_image" "postgres" {
	name = "postgres"
}

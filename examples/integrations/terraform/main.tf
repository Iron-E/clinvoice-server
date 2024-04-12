terraform {
	required_providers {
		docker = {
			source = "kreuzwerker/docker"
			version = "~> 3.0"
		}
	}
}

module "postgres" {
	source = "./postgres"
}

module "winvoice-server" {
	source = "./winvoice-server"
}

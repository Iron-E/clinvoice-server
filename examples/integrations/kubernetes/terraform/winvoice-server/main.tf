terraform {
	required_providers {
		kubernetes = {
			source  = "hashicorp/kubernetes"
			version = "~> 2.0"
		}
	}
}

module "docker-winvoice-server" {
	source = "../../../docker/terraform/winvoice-server"
	build-args = {
		rust-version = "1.77.0"
	}
}

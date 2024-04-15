terraform {
	required_providers {
		kubernetes = {
			source  = "hashicorp/kubernetes"
			version = "~> 2.0"
		}
	}
}

module "docker-postgres" {
	source = "../../../docker/terraform/postgres"
	image-version = var.metadata.labels.version
}

resource "kubernetes_manifest" "cloudnative-pg" {
	manifest = {
		apiVersion = "postgresql.cnpg.io/v1"
		kind = "Cluster"
		metadata = merge(var.metadata, { name = "postgres" })
		spec = {
			instances = 3

			storage = {
				size = "1Gi"
			}
		}
	}
}

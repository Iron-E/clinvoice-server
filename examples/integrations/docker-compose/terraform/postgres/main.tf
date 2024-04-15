terraform {
	required_providers {
		docker = {
			source = "kreuzwerker/docker"
			version = "~> 3.0"
		}
	}
}

module "postgres" {
	source = "../../../docker/terraform/postgres/"
}

# postgres

resource "docker_network" "postgres" {
	name = "winvoice/db/postgres/network"
}

resource "docker_secret" "postgres-db-name" {
   name = "winvoice/db/postgres/config/db-name"
	data = base64encode(file("${var.config-path}/name.txt"))
}

resource "docker_secret" "postgres-password" {
   name = "winvoice/db/postgres/config/password"
	data = base64encode(file("${var.config-path}/password.txt"))
}

resource "docker_secret" "postgres-user" {
   name = "winvoice/db/postgres/config/user"
	data = base64encode(file("${var.config-path}/user.txt"))
}

resource "docker_volume" "postgres-data" {
	name = "winvoice/db/postgres/data"
}

resource "docker_service" "db" {
	name = "winvoice/db/postgres"

	task_spec {
		container_spec {
			image = module.postgres.image.image_id

			env = {
				POSTGRES_DB_FILE = "/run/secrets/db/name.txt"
				POSTGRES_PASSWORD_FILE = "/run/secrets/db/password.txt"
				POSTGRES_USER_FILE = "/run/secrets/db/user.txt"
			}

			healthcheck {
				test = ["CMD-SHELL", "pg_isready --dbname \"$(cat /run/secrets/db/name.txt)\" --username \"$(cat /run/secrets/db/user.txt)\""]
				interval = "10s"
				timeout = "5s"
				retries =  5
			}

			mounts {
				target = "/var/lib/postgres/data"
				source = docker_volume.postgres-data.name
				type = "volume"
			}

			secrets {
				file_name = "db/name.txt"
				secret_id = docker_secret.postgres-db-name.id
				secret_name = docker_secret.postgres-db-name.name
			}

			secrets {
				file_name = "db/password.txt"
				secret_id = docker_secret.postgres-password.id
				secret_name = docker_secret.postgres-password.name
			}

			secrets {
				file_name = "db/user.txt"
				secret_id = docker_secret.postgres-user.id
				secret_name = docker_secret.postgres-user.name
			}
		}

		networks_advanced {
			name = docker_network.postgres.id
		}

		restart_policy {
			condition = "always"
		}
	}

	endpoint_spec {
		ports {
			name = "winvoice/db/postgres"
			target_port = 5432
		}
	}
}

output "service" {
	description = "The (winvoice db) postgres docker service"
   value = docker_service.db
}

output "postgres" {
   value = module.postgres
   description = "(winvoice db) postgres resources"
}

output "winvoice-server" {
   value = module.winvoice-server
   description = "winvoice-server resources"
}

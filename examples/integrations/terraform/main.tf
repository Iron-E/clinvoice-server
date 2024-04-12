module "postgres" {
	source = "./postgres"
}

module "winvoice_server" {
	source = "./winvoice_server"
}

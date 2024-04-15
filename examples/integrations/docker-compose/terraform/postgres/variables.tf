variable "config-path" {
	description = <<-EOT
		Path to a directory containing the following files:

		| Filename       | Description                                                              |
		| :-             | :-                                                                       |
		| `name.txt`     | The name of the database where `winvoice-server` will store information. |
		| `user.txt`     | The username which `winvoice-server` will use to login to the database.  |
		| `password.txt` | The password which `winvoice-server` will use to login to the database.  |

	EOT

	type = string
   default = "default_value"
}

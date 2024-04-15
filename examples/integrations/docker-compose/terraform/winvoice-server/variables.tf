variable "build-args" {
	description = <<-EOT
		Build arguments for the winvoice-server docker image.

		| Name              | Default       | Description                       |
		| :--               | :--           | :--                               |
		| `group-id`        | `10001`       | The ID of the created group.      |
		| `rust-version`    | `1.76.0`      | The Rust version to compile with. |
		| `user-id`         | `$GID`        | The ID of the created user.       |
	EOT
	type = object({
		group-id = optional(number),
		rust-version = optional(string),
		user-id = optional(number),
	})
	default = {}
}

variable "target-platform" {
   description = "the platform to build the container for"
   type = string
	default = ""
}

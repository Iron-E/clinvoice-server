variable "metadata" {
	description = <<-EOT
		## `labels`

		Values for the [recommended labels](https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/).

		## `namespace`

		The namespace to use for all kubernetes resources created by this module
	EOT
	default = {}
   type = object({
		namespace = optional(string, "winvoice")
		labels = optional(object({
			component = optional(string, "backend")
			managed-by = optional(string, "terraform")
			name = optional(string, "winvoice-server")
			part-of = optional(string, "winvoice")
			version = optional(string, "0.6.2")
		}), {})
   })
}

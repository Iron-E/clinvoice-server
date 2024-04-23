# winvoice-server

## Docker

A [Dockerfile](./Dockerfile) is provided to run the application in an isolated environment. A database is not included in the image, but is required to start the server.

### Build

There are several options for building, depending on what you are comfortable with.

#### `docker build`

Run the following command:

```sh
docker build [--build-arg <arg>=<value> ...] [--tag <tag>] .
```

For example:

```sh
docker build --build-arg RUST_VERSION=1.75.0 --tag winvoice-server:latest .
```

##### Arguments

| Name           | Default  | Description                       |
| :--            | :--      | :--                               |
| `GID`          | `10001`  | The ID of the created group.      |
| `RUST_VERSION` | `1.76.0` | The Rust version to compile with. |
| `UID`          | `$GID`   | The ID of the created user.       |

#### `terraform`

Optionally, you can use [terraform](https://github.com/hashicorp/terraform) to set up the relevant containers:

```terraform
## main.tf
module "winvoice-server" {
	source = "path/to/winvoice-server"

	# optionally, specify build arguments
	build-args = {
		rust-version = "1.77.0"
	}
}

## extra config as necessary…
```

Then, in a shell of your choice:

```sh
terraform init # setup
terraform apply # create winvoice-server containers
terraform destroy # delete winvoice-server containers
```

### Usage

After building, run:

```sh
docker run -p <port> <image-name> [<winvoice-server-arg> ...]
```

> [!NOTE]
>
> Use `docker exec` if you want to run a command inside a container rather than an image.

For example, to print help info, do:

```sh
docker run -p 3000 \
	-t \ # tty
	--rm \ # remove after executing
	<image-name> \
	help # run `winvoice-server help`
```

## Compose

A [compose file](./compose.yaml) is provided to run the application. A postgres image is included in the configuration.

Run the following command:

```sh
docker compose up
```

### Configs

| Name                       | Path                            | Description                              |
| :-                         | :-                              | :-                                       |
| `server-permissions-model` | `server/permissions/model.conf` | The `--permissions-model` argument value |

### Environment Variables

| Name                   | Default        | Description                           |
| :-                     | :-             | :-                                    |
| `WINVOICE_SERVER_ADDR` | `0.0.0.0:3000` | The address to host the server on     |
| `WINVOICE_SERVER_GIT`  | `master`       | The `git` branch or tag to build from |

### Secrets

| Name                        | Path                                   | Description                                                                                   |
| :-                          | :-                                     | :-                                                                                            |
| `server-cors`               | `config/server/cors/`                  | CORS-related information. See below.                                                          |
| `db`                        | `config/db/`                           | Databse credentials. See below.                                                               |
| `server-permissions-policy` | `config/server/permissions/policy.csv` | The `--permissions-policy` argument value.                                                    |
| `server-ssl`                | `config/server/ssl/`                   | SSL certificates. See below.                                                                  |
| `server-ssl-cadir`          | `config/server/ssl-cadir/`             | Trust authorities to use within the container. Structured like `/etc/ssl/certs/` in `alpine`. |

#### `db`

| Filename       | Description                                                              |
| :-             | :-                                                                       |
| `name.txt`     | The name of the database where `winvoice-server` will store information. |
| `user.txt`     | The username which `winvoice-server` will use to login to the database.  |
| `password.txt` | The password which `winvoice-server` will use to login to the database.  |

#### `server-cors`

| Filename    | Description                                              |
| :-          | :-                                                       |
| `allow.txt` | Corresponds to the `--cors-allow-origin` argument value. |

#### `server-ssl`

| Filename   | Description                                        |
| :-         | :-                                                 |
| `cert.pem` | Corresponds to the `--certificate` argument value. |
| `key.pem`  | Corresponds to the `--key` argument value.         |

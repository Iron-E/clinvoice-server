# winvoice-server

An example kubernetes configuration is provided to run the application. A postgres image is included in the configuration.

## Requirements

* `cloudnative-pg`
* `kubernetes`

> [!NOTE]
>
> There is a nix flake which can install these for you. Simply run `nix develop` inside this repository.

## Build

First, initialize cloudnative-pg:

```sh
kubectl cnpg install generate --watch-namespace example | kubectl apply --server-side -f -
```

Then, create a TLS cert and key (e.g. with [`mkcert`](https://github.com/FiloSottile/mkcert), also included in the flake), and then:

```sh
ktl create -n example secret tls winvoice.backend.tls --key key.pem --cert cert.pem
```

Finally, apply the configuration:

```sh
kubectl apply --recursive -f . # or the path to the kubernetes examples
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

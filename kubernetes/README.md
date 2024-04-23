# winvoice-server

An example kubernetes configuration is provided to run the application. A postgres image is included in the configuration.

## Requirements

* `cloudnative-pg`
* `kubectl`

> [!TIP]
>
> There is a nix [flake] which can install the above for you. Simply run `nix develop` inside this repository.

If you don't already have the `winvoice-server:0.6.3` docker image, follow [this guide](../README.Docker.md).

> [!IMPORTANT]
>
> If you are using `kind` (included in the [flake]), there is one extra step:
>
> ```sh
> kind load docker-image --name <your-cluster-name> winvoice-server:0.6.3
> ```
>
> This will bring `winvoice-server:0.6.3` into the scope for your `kind` cluster.

## Build

First, initialize cloudnative-pg:

```sh
kubectl cnpg install generate --watch-namespace example | kubectl apply --server-side -f -
```

Then, create a TLS cert and key (e.g. with [`mkcert`](https://github.com/FiloSottile/mkcert), also included in the [flake]), and then:

```sh
ktl create -n example secret tls winvoice.tls --key key.pem --cert cert.pem # the certificates
```

Finally, apply the configuration:

```sh
kubectl apply --recursive -f . # or the path to the kubernetes examples
```

[flake]: ../flake.nix

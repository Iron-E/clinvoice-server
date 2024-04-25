# winvoice-server

An example kubernetes configuration is provided to run the application. A postgres image is included in the configuration.

## Requirements

* `cloudnative-pg`
* `kubectl`

> [!NOTE]
>
> There is a nix [flake] which can install the above for you. Simply run `nix develop` inside this repository.

If you don't already have the `winvoice-server:0.6.4` docker image, follow [this guide](../README.Docker.md).

> [!IMPORTANT]
>
> If you are using `kind` (included in the [flake]), there are a few extra steps:
>
> ```sh
> kind create cluster --name winvoice --config ./kubernetes/kind/cluster.yaml
> kind load docker-image --name winvoice winvoice-server:0.6.4
> ```
>
> This will bring `winvoice-server:0.6.4` into the scope for your `kind` cluster.

## Build

First, initialize cloudnative-pg:

```sh
kubectl cnpg install generate --watch-namespace example | kubectl apply --server-side -f -
```

Then, initialize the nginx ingress controller based on the provider you are using (the following example is for `kind`):

```sh
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.10.1/deploy/static/provider/kind/deploy.yaml
```

Then, create a TLS cert and key (e.g. with [`mkcert`](https://github.com/FiloSottile/mkcert), also included in the [flake]), and then:

```sh
ktl create -n example secret tls winvoice.backend.tls --key key.pem --cert cert.pem # the certificates
ktl create -n example secret generic winvoice.backend.tls.cadir --from-file ssl-cadir/ # a CA dir which trusts the certificates
```

Finally, apply the configuration:

```sh
kubectl apply --recursive -f . # or the path to the kubernetes examples
```

[flake]: ../flake.nix

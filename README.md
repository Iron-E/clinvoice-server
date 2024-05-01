# winvoice-server

<!-- cargo-rdme start -->

`winvoice-server` is a backend for Winvoice libraries. It aims to allow any
number of different frontends, such as [winvoice-cli](https://github.com/Iron-E/winvoice-cli) or
[winvoice-gui](https://github.com/Iron-E/winvoice-gui), to communicate with it without having to be
written in Rust or re-implement common procedures.

## Usage

See `winvoice-server help`.

> **Note**
>
> A template user will be created upon first running the user. It has the following fields (some information omitted
> for simplicity):
>
> ```json
> {
>   "username": "admin",
>   "password": "password",
>   "role": {
>     "name": "admin"
>     "password_ttl": null, # password lasts forever
>   },
> }
> ```
>
> It is recommended to change the password and the Password TTL of the admin role in order to increase the security
> of your installation.

### Installation

Requirements:

* [`cargo`](https://github.com/rust-lang/cargo)

```sh
cargo install \
  --features <adapters> \
  --git https://github.com/Iron-E/winvoice-server \
  --root=<desired install folder>
```

### API

You can add `winvoice-server` to your `[dependencies]` to access the `winvoice_server::api`
directly:

```toml
[dependencies.winvoice-server]
branch = "master"
default-features = false
git = "https://github.com/Iron-E/winvoice-server"
```

To see information about the API, run `cargo doc` and look for `winvoice_server::api::request`,
`winvoice_server::api::response`, and `winvoice_server::api::routes`.

## Development

### Self-signed certificates

I recommend the use of the tool [`mkcert`](https://github.com/FiloSottile/mkcert) to generate trusted certificates
on your local machine, for the purposes of writing a front-end.

<!-- cargo-rdme end -->

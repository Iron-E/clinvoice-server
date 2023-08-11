# winvoice-server

<!-- cargo-rdme start -->

`winvoice-server` is WIP backend for Winvoice libraries. It aims to allow any
number of different frontends, such as [winvoice-cli](https://github.com/Iron-E/winvoice-cli) or
[winvoice-gui](https://github.com/Iron-E/winvoice-gui), to communicate with it without having to be
written in Rust or re-implement common procedures.

## Usage

* For basic information, run `winvoice-server help` from the command line.
* For an in-depth guide, see the [wiki](https://github.com/Iron-E/winvoice-server/wiki/Usage).

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

If you are working with another language, see [the docs](TODO).

## Development

### Self-signed certificates

I recommend the use of the tool [`mkcert`](https://github.com/FiloSottile/mkcert) to generate trusted certificates
on your local machine, for the purposes of writing a front-end.

<!-- cargo-rdme end -->

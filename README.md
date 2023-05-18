# Winvoice

<!-- cargo-rdme start -->

`winvoice-server` is WIP backend for Winvoice libraries. It aims to allow any
number of different frontends, such as [Winvoice](https://github.com/Iron-E/winvoice) or
[GUInvoice](https://github.com/Iron-E/guinvoice), to communicate with it without having to be
written in Rust or re-implement common procedures.

## Installation

Requirements:

* [`cargo`](https://github.com/rust-lang/cargo)

```sh
cargo install \
  --features <adapters> \
  --git https://github.com/Iron-E/winvoice-server \
  --root=<desired install folder>
```

## Usage

* For basic information, run `winvoice-server help` from the command line.
* For an in-depth guide, see the [wiki](https://github.com/Iron-E/winvoice-server/wiki/Usage).

<!-- cargo-rdme end -->

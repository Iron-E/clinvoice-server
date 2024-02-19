# winvoice-server

## Build

Run the following command:

```sh
docker build [--build-arg <arg>=<value> ...] [--tag <tag>] .
```

For example:

```sh
docker build --build-arg PORT=3000 --build-arg RUST_VERSION=1.75.0 --tag winvoice-server:latest .
```

#### Arguments

| Name           | Default  | Description                        |
| :--            | :--      | :--                                |
| `PORT`         | `3000`   | The port the server will listen to |
| `RUST_VERSION` | `1.76.0` | The Rust version to compile with.  |

#### Environment Variables

## Usage

After building, run:

```sh
docker run <image-name> [<winvoice-server-arg> ...]
```

For example, to print help info, do:

```sh
docker run \
	-t \ # tty
	--rm \ # remove after executing
	<image-name> \
	help # run `winvoice-server help`
```

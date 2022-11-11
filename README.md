# Ohm wallet

A GRPC based cosiging service.

### Stack

- Rust >= 1.56
- SQLite >= 3.35
- Diesel >= 2.0.0
- BDK >= 0.23.0

## Usage 

### Initialise DB

```bash
cargo install diesel_cli --no-default-features --features sqlite
diesel --database-url ohm.sqlite migration run
```

### Start GRPC server

```bash
ohm-server -c ohm.cfg_example
```

### Manage cosigners

```
USAGE:
    ohm-client cosigner <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    find
    forget
    help        Prints this message or the help of the given subcommand(s)
    info
    register
```

### Manage wallets

```
ohm-client-wallet 0.1.0

USAGE:
    ohm-client wallet <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    create
    find
    forget
    help      Prints this message or the help of the given subcommand(s)
    info
```
### Manage PSBTs

```
USAGE:
    ohm-client psbt <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    broadcast
    combine
    create
    find
    forget
    help         Prints this message or the help of the given subcommand(s)
    info
    register
    sign
```

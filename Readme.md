# OParl Validator

This repository contains an [OParl](https://oparl.org/) validator that checks for common validity problems in OParl implementation. It's meant to be used through the CLI, but there's also an experimental web frontend.

## Installation

You can download binaries for windows, mac and linux at https://konstin.github.io/oparl-validator-rs/.

To build from source, [install rust](https://rustup.rs/) and run

```shell
cargo install --git https://github.com/konstin/oparl-validator-rs
```

## Usage

```shell
oparl-validator-rs <endpoint url>
```

This will write a report to `report.txt`. Note that most endpoints are slow so this can easily take more than an hour . There is also a `--cache` option to improve performance and reduce server load with multiple runs. Using `all` as endpoint url validates all of [endpoints.yml](https://github.com/OParl/resources/blob/main/endpoints.yml), writing a report for each endpoint.

## Web frontend

You can build the experimental web frontend with

```shell
npm ci
npm run build
```

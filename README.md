# `spin-pluginify` - a Spin plugin to help build Spin plugins

This is a [Spin](https://developer.fermyon.com/spin/index) plugin that helps with the inner loop of Spin plugin development by creating the tar file and manifest for you.

## Installation

```
cargo build --release
./target/release/spin-pluginify
spin plugins install --file spin-pluginify.json --yes
```

## Usage

### Preparation

For your plugin, create a `spin-pluginify.toml` file with the following content:

```toml
name = "<PLUGIN-NAME>"
version = "0.1"
spin_compatibility = ">=0.7"
license = "Apache-2.0"
package = "<./PATH/TO/EXECUTABLE>"
```

You can find examples in this repo and in https://github.com/itowlson/spin-trigger-sqs-poc.

### Usage

When you have a new build of your plugin ready:

* Run `spin pluginify`
  * It should create or update a `.tar.gz` file and a `<PLUGIN-NAME>.json` manifest
* Run `spin plugins install --file <PLUGIN-NAME>.json --yes`

Your plugin should then be installed in Spin and ready to test.

## Troubleshooting

Error handling is non-existent right now so, uh, sorry.

# Spin `pluginify` - a Spin plugin to help build Spin plugins

This is a [Spin](https://developer.fermyon.com/spin/index) plugin that helps with the inner loop of Spin plugin development by creating the tar file and manifest for you.

## Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) if building from source
* [Spin](https://developer.fermyon.com/spin/install)

# Installation

Latest release:

```bash
spin plugins install pluginify
```

From source:

```bash
git checkout https://github.com/itowlson/spin-pluginify
cd spin-pluginify
cargo build --release
./target/release/pluginify --install
```

# Usage

## Preparation

For your plugin, create a `spin-pluginify.toml` file with the following content:

```toml
name = "<PLUGIN-NAME>"
version = "0.1"
spin_compatibility = ">=0.7"
license = "Apache-2.0"
package = "<./PATH/TO/EXECUTABLE>"
# optional - if present these files will be added to the plugin tar file
assets = [ "path/to/asset/1", "path/to/asset/2" ]
```

You can find examples in this repo and in https://github.com/fermyon/spin-trigger-sqs.

## Packaging and installing your plugin

When you have a new build of your plugin ready, run:

```bash
spin pluginify --install
```

Your plugin should then be installed in Spin and ready to test.

Alternatively, you can just package by running `spin pluginify`. This creates (or updates) a `<PLUGIN-NAME>.tar.gz` file and a `<PLUGIN-NAME>.json` manifest. You can then run `spin plugins install --file <PLUGIN-NAME>.json --yes` to install that package into Spin.

## "I want it NOW" mode

To install an executable as a Spin plugin without creating a `spin-pluginify.toml`, run `spin pluginify --immediate <PATH/TO/EXECUTABLE> --install`.

(This is great for one-off experiments, but if you're iterating on the plugin then create a `spin-pluginify.toml` and save yourself typing the path all the time eh.)

# Troubleshooting

Error handling is a bit patchy at the moment. Please raise an issue if you find an error which is cryptic, or panicky, or shouldn't be an error at all!

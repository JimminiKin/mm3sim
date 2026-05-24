#!/bin/bash
set -euo pipefail

# fix home issues
export HOME=/root

# Install Rustup
if ! command -v rustup
then
  echo "Installing Rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -t wasm32-unknown-unknown --profile minimal
  source "$HOME/.cargo/env"
else
  echo "Rustup already installed."
fi

rustup target add wasm32-unknown-unknown

# Install trunk using binstall
if ! command -v trunk
then
  echo "Installing trunk with binstall..."
  curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
  cargo binstall --targets x86_64-unknown-linux-musl -y trunk
  echo "Trunk installation complete."
else
  echo "trunk already installed."
fi

# Install trunk using binstall
if ! command -v wasm-bindgen
then
  echo "Installing wasm-bindgen-cli ..."
  cargo install wasm-bindgen-cli
  echo "wasm-bindgen-cli installation complete."
else
  echo "wasm-bindgen-cli already installed."
fi


# Build project with trunk
echo "Building project with trunk"
trunk build

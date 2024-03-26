#!/bin/sh

# This script adds Rokit to PATH if it is not already there. It is adapted from Rustup:
# https://github.com/rust-lang/rustup/blob/d33c53f0d1aac036b7d76c4b6ff812f3f5b00240/src/cli/self_update/env.sh

case ":${PATH}:" in
    *:"{rokit_bin_path}":*)
        ;;

    *)
        export PATH="{rokit_bin_path}:$PATH"
        ;;
esac

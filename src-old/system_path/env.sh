#!/bin/sh

# This script adds Aftman to the user's PATH and is run via the user's
# shell-specific profile.
#
# This is adapted from Rustup:
# https://github.com/rust-lang/rustup/blob/feec94b6e0203cb6ad023b1e2c953d058e5c3acd/src/cli/self_update/env.sh

case ":${PATH}:" in
    *:"{our_bin_dir}":*)
        ;;

    *)
        export PATH="{our_bin_dir}:$PATH"
        ;;
esac
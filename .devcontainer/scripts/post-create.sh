#!/bin/bash

setup_rust(){
    echo "adding target i686-unknown-linux-gnu"
    rustup target add i686-unknown-linux-gnu
}

main(){
    setup_rust
}

main "$@"
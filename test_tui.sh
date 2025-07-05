#!/bin/bash
RUST_BACKTRACE=1 cargo run -- debug $1 2> log.txt
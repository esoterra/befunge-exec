#!/bin/bash
RUST_BACKTRACE=1 cargo flamegraph -o flamegraph.svg -- tui ./programs/babyshark.b93 2> log.txt
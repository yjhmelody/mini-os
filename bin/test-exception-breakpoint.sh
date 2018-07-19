#!/usr/bin/env bash

## when it works, it prints some info, when it failed, it will print "failed" with some info
bootimage run --bin test-exception-breakpoint -- \
    -serial mon:stdio -display none \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04
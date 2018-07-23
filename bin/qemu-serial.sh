#!/usr/bin/env bash

qemu-system-x86_64 \
-drive format=raw,file=target/x86_64-mini_os/debug/bootimage-mini_os.bin \
-serial mon:stdio \
-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
-display none

#!/bin/bash
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_PATH" || exit

dd if=$1 of=installer.img bs=4M # Map Installer

dd if=/dev/zero of=$IMG bs=1M count=40000 # 40 Gb Image Mapping

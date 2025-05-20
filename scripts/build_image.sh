#!/bin/bash

load_addr=0x40200000
entry_point=0x40200000
output_name=X-Hyper_Uimage
image_name=X-Hyper

if [ -d "./build" ]; then
    rm -rf ./build
fi

mkdir -p build
cd build

cmake ..
make

# mkimage
mkimage -A arm64 -O linux -C none -a $load_addr -e $entry_point -d ${image_name} ${output_name}
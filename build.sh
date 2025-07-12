#!/bin/bash

cargo build --release && \
rm -rf build/partydeck-rs
mkdir -p build/ build/res && \
cp target/release/partydeck-rs res/PartyDeckKWinLaunch.sh build/ && \
cp res/splitscreen_kwin.js res/splitscreen_kwin_vertical.js build/res && \
if [ -d res/gamescope ]; then cp -r res/gamescope build/res; fi && \
libpath=$(find target/release/build -name libsteam_api.so | head -n 1) && \
cp "$libpath" build/

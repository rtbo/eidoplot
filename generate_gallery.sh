#!/bin/sh

cargo run --example bars -- png=gallery/bars.png
cargo run --example gauss -- png=gallery/gauss.png
cargo run --example sine -- png=gallery/sine.png
cargo run --example bouncing_ball --features time -- png=gallery/bouncing_ball.png

cargo run --example bode_rlc --features noto-serif-italic,utils -- png=gallery/bode_rlc.png svg=gallery/bode_rlc.svg
cargo run --example bode_rlc --features noto-serif-italic,utils -- png=gallery/bode_rlc_macchiato.png macchiato
cargo run --example bode_rlc --features noto-serif-italic,utils -- png=gallery/bode_rlc_mocha.png mocha

cargo run --example multiple_axes --features utils -- png=gallery/multiple_axes.png
cargo run --example subplots --features utils -- png=gallery/subplots.png

cargo run --example bitcoin --features data-csv,time -- png=gallery/bitcoin.png
cargo run --example iris --features data-csv -- png=gallery/iris.png


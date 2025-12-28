#! /bin/bash

RUN_POLARS=n
POS_ARGS=()

while [[ $# -gt 0 ]]; do
        case $1 in
        -p | --polars)
                RUN_POLARS=y
                shift
                ;;
        -* | --*)
                echo "Unknown option $1"
                shift
                ;;
        *)
                POS_ARGS+=("$1")
                shift
                ;;
        esac
done

cargo run --example text_line --package eidoplot-text --features noto-sans
cargo run --example text_rich --package eidoplot-text --features noto-sans,noto-serif

cargo run --example bars -- ${POS_ARGS[@]}
cargo run --example gauss -- ${POS_ARGS[@]}
cargo run --example sine -- ${POS_ARGS[@]}

cargo run --example bouncing_ball --features time -- ${POS_ARGS[@]}

cargo run --example bode_rlc --features noto-serif-italic,utils -- ${POS_ARGS[@]}

cargo run --example multiple_axes --features utils -- ${POS_ARGS[@]}
cargo run --example subplots --features utils -- ${POS_ARGS[@]}

cargo run --example bitcoin --features data-csv,time -- ${POS_ARGS[@]}
cargo run --example iris --features data-csv -- ${POS_ARGS[@]}

cargo run --example bode_rlc_eplt --features eplt,noto-serif-italic,utils -- ${POS_ARGS[@]}

cargo run --example iris_eplt --features data-csv,eplt -- ${POS_ARGS[@]}

cargo run --example multiple_axes_eplt --features eplt,utils -- ${POS_ARGS[@]}
cargo run --example subplots_eplt --features eplt,utils -- ${POS_ARGS[@]}

if [ "$RUN_POLARS" = "y" ]; then
        cargo run --example polars_iris --features data-polars -- ${POS_ARGS[@]}
        cargo run --example polars_sine --features data-polars -- ${POS_ARGS[@]}
fi

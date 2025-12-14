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

cargo run --example text_line
cargo run --example text_rich

cargo run --example bars -- ${POS_ARGS[@]}
cargo run --example bitcoin -- ${POS_ARGS[@]}
cargo run --example bode_rlc -- ${POS_ARGS[@]}
cargo run --example bouncing_ball -- ${POS_ARGS[@]}
cargo run --example gauss -- ${POS_ARGS[@]}
cargo run --example iris -- ${POS_ARGS[@]}
cargo run --example multiple_axes -- ${POS_ARGS[@]}
cargo run --example sine -- ${POS_ARGS[@]}
cargo run --example subplots -- ${POS_ARGS[@]}

cargo run --example bode_rlc_eplt --features dsl-diag -- ${POS_ARGS[@]}
cargo run --example iris_eplt --features dsl-diag -- ${POS_ARGS[@]}
cargo run --example multiple_axes_eplt --features dsl-diag -- ${POS_ARGS[@]}
cargo run --example subplots_eplt --features dsl-diag -- ${POS_ARGS[@]}

if [ "$RUN_POLARS" = "y" ]; then
        cargo run --example polars_iris --features data-polars -- ${POS_ARGS[@]}
        cargo run --example polars_sine --features data-polars -- ${POS_ARGS[@]}
fi

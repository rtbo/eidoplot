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
cargo run --example gauss -- ${POS_ARGS[@]}
cargo run --example iris -- ${POS_ARGS[@]}
cargo run --example sine -- ${POS_ARGS[@]}

cargo run --example iris_eplt --features dsl-diag -- ${POS_ARGS[@]}
cargo run --example rlc_bode --features dsl-diag -- ${POS_ARGS[@]}
cargo run --example subplots --features dsl-diag -- ${POS_ARGS[@]}

if [ "$RUN_POLARS" = "y" ]; then
        cargo run --example polars_iris --features polars -- ${POS_ARGS[@]}
        cargo run --example polars_sine --features polars -- ${POS_ARGS[@]}
fi

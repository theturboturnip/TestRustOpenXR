run:
    cargo run --features=desktop

run-log:
    cargo run --features=desktop 2>&1 | tee just-run.log


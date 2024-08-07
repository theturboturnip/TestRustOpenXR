default:
    @just --list

run:
    cargo run --features=desktop

run-log:
    cargo run --features=desktop 2>&1 | tee just-run.log

compile-shader name:
    glslc --target-env=vulkan1.1 ./src/shaders/glsl/{{name}} -o ./src/shaders/spv/{{name}}.spv
    spirv-dis ./src/shaders/spv/{{name}}.spv > ./src/shaders/disasm/{{name}}.disasm

compile-shaders:
    just compile-shader fullscreen.vert
    just compile-shader debug_pattern.frag
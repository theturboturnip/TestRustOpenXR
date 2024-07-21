run:
    cargo run --features=desktop

run-log:
    cargo run --features=desktop 2>&1 | tee just-run.log

compile-shaders:
    glslc --target-env=vulkan1.1 ./src/shader.glsl.vert -o ./src/fullscreen.vert.spv
    spirv-dis ./src/fullscreen.vert.spv > ./src/fullscreen.vert.spv.disasm
    glslc --target-env=vulkan1.1 ./src/shader.glsl.frag -o ./src/debug_pattern.frag.spv
    spirv-dis ./src/debug_pattern.frag.spv > ./src/debug_pattern.frag.spv.disasm
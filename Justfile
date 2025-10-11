set dotenv-load

default:
    @just --list

prepare-android:
    rustup target add aarch64-linux-android
    cargo install cargo-ndk

run-quest:
    cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build
    ./gradlew build
    ./gradlew installDebug
    adb shell setprop debug.oculus.loadandinjectpackagedvvl.co.realfit.naopenxrwgpu 1

run-local:
    cargo run --features=desktop

run-local-log:
    cargo run --features=desktop 2>&1 | tee just-run.log

compile-shader name:
    glslc --target-env=vulkan1.1 ./src/shaders/glsl/{{name}} -o ./src/shaders/spv/{{name}}.spv
    spirv-dis ./src/shaders/spv/{{name}}.spv > ./src/shaders/disasm/{{name}}.disasm

compile-shaders:
    just compile-shader fullscreen.vert
    just compile-shader debug_pattern.frag
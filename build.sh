set -ex

cd wave-function-collapse
wasm-pack build --target web

cd ..
cp "wave-function-collapse/pkg/wave_function_collapse_bg.wasm" demo
cp "wave-function-collapse/pkg/wave_function_collapse.js" demo
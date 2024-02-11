Implementation of wave function collapse in rust/wasm

# Requirements

- rust
- wasm-pack (`cargo install wasm-pack`)

# Compile

`./build.sh`

Runs wasm-pack and copy files to demo dir  
Then
`python3 -m http.server --directory demo/`

To run tests:
`cd wave-function-collapse && cargo test`


# Notes

Tile adjency constraints is a list containing:
[tile_id, axis, direction, excluded_tile_id]

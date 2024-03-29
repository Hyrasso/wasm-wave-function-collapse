Implementation of wave function collapse in rust/wasm

Demo page: https://hyrasso.github.io/wasm-wave-function-collapse/demo/

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
[tile_id, axis, direction, allowed_tile_id]

# TODO

- [x] Take constraints from JS
- [X] Change constraints to allowed neighbor tile instead of forbidden (easier to add tiles without having it show up at random places)
- [ ] Check that step returns false when collapsing an impossible state
- [X] Set position dimension size to 3 in all cases (with the allowing neighbors formulation 1d/2d work the same, just ignoring an axis)
- [ ] Allow to not return all the dimensions
- [ ] Better error reporting to Js (serde might be returning Err that could be returned to js directly)
- [ ] Better demo
 - [X] 2d version
 - [ ] Better filling of the space
- [ ] Check if we actually need the full entropy computation for the tile ordering, maybe just state * tile weight is enough?
- [ ] Helpers for easier tiles constraints generation, one way could be have an adjency type  
    eg: (axis N, direction, connection id), then we can get allowed neighbors by checking other tiles conn id at axis N, in the opposite direction
- [ ] Implement the NxN overlap model using this formulation
- [ ] Better coverage of tests
- [ ] Set the number of tiles in the type in some way
- [ ] Switch state vec to usize instead, or hashset, instead of f64  
    or consider having a continuous state instead of binary? I guess that could impact the entropy computation, but in the end we still want to allow any proba > 0
- [ ] Remove the global instance thing  
    Either have the object shared with js (check if that would impact performance), or allow multiple instances instead of only one, maybe share a dummy object which is just an id with the methods  
- [ ] Allow bounds on the generation space
- [ ] Allow wrapping generation space
- [ ] Add backtracking in case of generation failure (not sure what would be the interface for that)
- [ ] Have a step function that also returne the new tile
- [ ] State proba distribution readout for wavefront/a given area
- [ ] Additional generation constraints (eg forbid loops, force global connectivity)

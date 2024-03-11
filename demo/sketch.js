import init, { init_instance, step, read_state } from './wave_function_collapse.js';


const mainSketch = (p) => {
  // 0    1    2    3    4
  // .#.  ...  ...  .#.  ...
  // .##  .##  ##.  ##.  ...
  // ...  .#.  .#.  ...  ...
  // axis 1:x, 2:y, direction: 1:rigt, 1:down
  const tile_connectors = [
    {tid: 0, axis: 0, dir: 1, cid: 1},
    {tid: 0, axis: 0, dir: -1, cid: 0},
    {tid: 0, axis: 1, dir: 1, cid: 0},
    {tid: 0, axis: 1, dir: -1, cid: 1},
    {tid: 1, axis: 0, dir: 1, cid: 1},
    {tid: 1, axis: 0, dir: -1, cid: 0},
    {tid: 1, axis: 1, dir: 1, cid: 1},
    {tid: 1, axis: 1, dir: -1, cid: 0},
    {tid: 2, axis: 0, dir: 1, cid: 0},
    {tid: 2, axis: 0, dir: -1, cid: 1},
    {tid: 2, axis: 1, dir: 1, cid: 1},
    {tid: 2, axis: 1, dir: -1, cid: 0},
    {tid: 3, axis: 0, dir: 1, cid: 0},
    {tid: 3, axis: 0, dir: -1, cid: 1},
    {tid: 3, axis: 1, dir: 1, cid: 0},
    {tid: 3, axis: 1, dir: -1, cid: 1},
    {tid: 4, axis: 0, dir: 1, cid: 1},
    {tid: 4, axis: 0, dir: -1, cid: 1},
    {tid: 4, axis: 1, dir: 1, cid: 1},
    {tid: 4, axis: 1, dir: -1, cid: 1},
  ];
  const gen_constraints = (tile_connectors) => {
    const connectors_map = {};
    const constraints = [];
    for (let {tid: tid, axis: axis, dir: dir, cid: cid} of tile_connectors) {
      if (!(axis in connectors_map)) {
        connectors_map[axis] = {};
      }
      if (!(dir in connectors_map[axis])) {
        connectors_map[axis][dir] = {};
      }
      if (tid in connectors_map[axis][dir]) {
        console.warn("Unexpected duplicate");
      }
      connectors_map[axis][dir][tid] = cid;
    }
    for (let {tid: tid, axis: axis, dir: dir, cid: cid} of tile_connectors) {
      let ntile = connectors_map[axis][-dir];
      for (let [ntid, ncid] of Object.entries(ntile)) {
        if (ncid == cid) {
          constraints.push([tid, axis, dir, parseInt(ntid)]);
        }
      }
    }
    return constraints;
  }
  const init_constraints = gen_constraints(tile_connectors);
  //let res = [];
  //for (let c of init_constraints) {
  //  res.push(`${c[0]}, ${c[1]}, ${c[2]}, ${c[3]}`)
  //}
  //console.log(init_constraints.join("\n"));
  console.log(init_constraints);
  const tiles_weights = [1.0, 1.0, 1.0, 1.0, 0.5];
  const NTILE = 40
  let loaded = false

  const params = new Proxy(new URLSearchParams(window.location.search), {
    get: (searchParams, prop) => searchParams.get(prop),
  });
  let seed = parseInt(params.seed) || Math.ceil(Math.random() * 0xFFFFFFFF);

  let bound_min_x, bound_max_x, bound_min_y, bound_max_y

  p.preload = async () => {
    await init();
    // console.log(init_constraints);
    console.log("Seed", seed);
    init_instance(init_constraints, tiles_weights, seed);
    loaded = true;
  }

  p.setup = () => {
    p.createCanvas(window.innerWidth, window.innerHeight);
    p.background(200);
    bound_min_x = 0;
    bound_min_y = 0;
    bound_max_x = 0;
    bound_max_y = 0;
  }

  let total_step_count = 0;
  
  let paused = false;
  p.keyReleased = () => {
    if (p.key == " ") {
      if (paused) {
        p.loop();
      } else {
        p.noLoop();
      }
      paused = !paused;
    }
  }
  p.draw = () => {
    p.background(128);
    if (!loaded) {
      return;
    }

    // p.translate(-bound_min_x + p.width / 2, -bound_min_y + p.height / 2);
    let scalex = p.width / (bound_max_x - bound_min_x);
    let scaley = p.height / (bound_max_y - bound_min_y);
    let scale = p.min(scalex, scaley, 1);
    // scale = scale - 0.2;
    // scale = 1;
    if (!step()) {
      p.noLoop();
      console.log("Step failed to complete")
    }
    // if (total_step_count > p.width / NTILE * p.height / NTILE) {
    //   console.log("Canva should be somewhat filled");
    //   p.noLoop();
    //   speed_test();
    // }
    total_step_count += 1;
    let state = read_state();
    console.log(state);
    let w = p.width / NTILE;
    p.textAlign(p.CENTER, p.CENTER);
    p.textSize(w);
    p.noFill();
    let tiles = "└┌┐┘┼".split("");
    for (let [x, y, _z, _w, tid] of state) {
      bound_max_x = p.max(bound_max_x, x * w + w);
      bound_max_y = p.max(bound_max_y, y * w + w);
      bound_min_x = p.min(bound_min_x, x * w);
      bound_min_y = p.min(bound_min_y, y * w);
    }
    for (let [x, y, _z, _w, tid] of state) {
      p.noFill();
      p.rect((x*w - bound_min_x) * scale, (y*w - bound_min_y) * scale, w * scale, w * scale);
      p.fill(255);
      p.text(tiles[tid], ((x * w + w / 2) - bound_min_x) * scale, ((y * w + w / 2) - bound_min_y) * scale);
      // if (tid == 0) {
      //   p.text("L", x * w + w / 2, y * w + w / 2);
      // } else if (tid >= 1) {
      //   p.text(`${tid}`, x * w + w / 2, y * w + w / 2);
      // }
      if (x < -NTILE || x > NTILE) {
        p.noLoop();
        // quick speed test
        speed_test();
      }
    }

    function speed_test() {
      let t1 = window.performance.now();
      let f = true;
      let N = 10000;
      for (let i = 0; i < N; i += 1) {
        f &= step();
      }
      console.log((window.performance.now() - t1) / N, "ms/it", f);
    }
  }
  // p.noLoop();
  // return;
}

new p5(mainSketch);
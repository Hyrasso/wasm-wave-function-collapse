import init, { init_instance, step, read_state } from './wave_function_collapse.js';


const mainSketch = (p) => {

  const init_constraints = [
    [0, 0, -1, 1],
    [0, 0, -1, 0],
    [0, 0, 1, 0],
    [0, 0, 1, 3],
    [1, 0, -1, 2],
    [1, 0, 1, 0],
    [1, 0, -1, 3],
    [1, 0, 1, 3],
    [2, 0, -1, 2],
    [2, 0, 1, 1],
    [2, 0, 1, 2],
    [2, 0, -1, 3],
    [3, 0, -1, 0],
    [3, 0, -1, 1],
    [3, 0, 1, 1],
    [3, 0, 1, 2],
  ];
  const tiles_weights = [1.0, 1.0, 1.0, 10.0];
  const NTILE = 40
  let loaded = false
  p.preload = async () => {
    await init();
    // console.log(init_constraints);
    init_instance(init_constraints, tiles_weights, 41);
    loaded = true;
  }
  p.setup = () => {
    p.createCanvas(400, 400);
    p.background(200);
  }

  p.draw = () => {
    p.background(128);
    if (!loaded) {
      return;
    }
    p.translate(p.width / 2, p.height / 2)
    if (!step()) {
      p.noLoop();
    }
    let state = read_state();
    console.log(state);
    let w = p.width / NTILE;
    p.noStroke();
    for (let [x, y, _z, _w, tid] of state) {
      if (tid == 0) {
        p.circle(x * w + w / 2, 0, w);
        p.rect(x * w + w / 2, - w/2, w/2, w);
      } else if (tid == 1) {
        p.rect(x * w, - w/2, w, w);
      } else if (tid == 2) {
        p.circle(x * w + w / 2, 0, w);
        p.rect(x * w, - w/2, w/2, w);
      } else if (tid == 3) {
        // pass
      }
      if (x < -NTILE || x > NTILE) {
        p.noLoop();
        // quick speed test
        let t1 = window.performance.now()
        let f = true;
        let N = 100000;
        for (let i=0;i < N;i+=1) {
          f &= step();
        }
        console.log((window.performance.now() - t1) / N, "ms/it", f);
      }
    }
  }
}

new p5(mainSketch);
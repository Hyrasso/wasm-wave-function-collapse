import init, { init_instance, step, read_state } from './wave_function_collapse.js';


const mainSketch = (p) => {
  const NTILE = 40
  let loaded = false
  p.preload = async () => {
    await init();
    init_instance();
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
    for (let [x, y, tid] of state) {
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
      }
    }
  }
}

new p5(mainSketch);
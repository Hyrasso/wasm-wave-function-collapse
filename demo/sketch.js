import init, { init_instance, step, read_state } from './wave_function_collapse.js';

async function run() {
    await init();
    console.log(init_instance());
    console.log(step(), read_state());
    console.log(step(), read_state());
}

run();

const mainSketch = (p) => {
  p.setup = () => {
    p.createCanvas(400, 400);
    p.background(200)
    p.rect(100, 100, 200, 200)
  }
//   p.draw = () => {
//     p.background(128);
//   }
}

new p5(mainSketch);
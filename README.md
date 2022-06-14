# Particles in wasm for the html canvas

You'll need [wasm-pack](https://github.com/rustwasm/wasm-pack) installed to build this project.

You can build it like so: `wasm-pack build -t web --release`.  
Then you take the wasm_particles_bg.wasm and wasm_particles.js and put it onto your webserver.  
Finally you can load it into an html canvas with the id of `canvas` with the following javascript in a script block (with `type="module"`)in your webpage. I recommend putting it towards the bottom so that everything else can load first.

```js
import init from './wasm_particles.js';
async function run() {
    await init('./wasm_particles_bg.wasm');
}
run();
```

It's a bit on the performance hungry side, but it works great on my machine :D

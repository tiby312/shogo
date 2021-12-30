import { default as init } from './pkg/demo.js';
var w=await init('pkg/demo_bg.wasm');
await w.worker_entry();

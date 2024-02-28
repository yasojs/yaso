import { EventEmitter } from "events";
import { addBenchmark, startBenchmarking } from "../benchmarker.mjs";

let event_emitter = new EventEmitter();

event_emitter.on("dummy", () => {});

let levels = [1, 5, 10, 25, 50, 100, 250, 500, 1000];

for (const level of levels) {
    addBenchmark("events", `emitter.emit('dummy') * ${level}`, () => {
        for (let i = 0; i < level; i++) {
            event_emitter.emit("dummy");
        }
    });
}

startBenchmarking();

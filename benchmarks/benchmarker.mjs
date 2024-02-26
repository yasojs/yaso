import process from "process";

function calculateFunctionExecuteTime(fn) {
    let start = Number(process.hrtime.bigint());

    fn();

    let end = Number(process.hrtime.bigint());

    let elapsed = end - start;

    return elapsed / 1_000_000;
}

class Benchamrk {
    constructor(category, title, fn) {
        this.category = category;
        this.title = title;
        this.fn = fn;
    }

    run() {
        console.log(`Running ${this.title} of ${this.category} benchmarks..`);

        let elapsed = calculateFunctionExecuteTime(this.fn);

        console.log(`Took ${elapsed}ms to run the benchamrk`);
        console.log();
    }
}

let benchamrks = [];

export function addBenchmark(category, title, fn) {
    benchamrks.push(new Benchamrk(category, title, fn));
}

export function startBenchmarking() {
    let elapsed = calculateFunctionExecuteTime(() => {
        for (const benchamrk of benchamrks) {
            benchamrk.run();
        }
    });

    console.log(`Took ${elapsed}ms to run all benchmarks`);
    console.log();
}

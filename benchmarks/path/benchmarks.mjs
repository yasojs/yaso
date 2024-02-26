import path from "path";
import { addBenchmark, startBenchmarking } from "../benchmarker.mjs";

addBenchmark("path", "path.parse('/usr/bin/yaso')", () => {
    path.parse("/usr/bin/yaso");
});

addBenchmark(
    "path",
    "path.format({ root: '/', dir: '/usr/bin', base: 'yaso', ext: '', name: 'yaso', })",
    () => {
        path.format({
            root: "/",
            dir: "/usr/bin",
            base: "yaso",
            ext: "",
            name: "yaso",
        });
    },
);

addBenchmark(
    "path",
    "path.normalize('.//./../...//path/benchmarks.mjs')",
    () => {
        path.normalize(".//./../...//path/benchmarks.mjs");
    },
);

addBenchmark(
    "path",
    "path.resolve('usr/bin/yaso', '../hyperfine', '../../lib/rustlib')",
    () => {
        path.resolve("usr/bin/yaso", "../hyperfine", "../../lib/rustlib");
    },
);

addBenchmark(
    "path",
    "path.join('/usr/bin/yaso', '../hyperfine', '../../lib/rustlib')",
    () => {
        path.join("/usr/bin/yaso", "../hyperfine", "../../lib/rustlib");
    },
);

addBenchmark("path", "path.dirname('../path/benchmarks.mjs')", () => {
    path.dirname("../path/benchmarks.mjs");
});

addBenchmark("path", "path.basename('../path/benchmarks.mjs')", () => {
    path.basename("../path/benchmarks.mjs");
});

addBenchmark("path", "path.extname('../path/benchmarks.mjs')", () => {
    path.extname("../path/benchmarks.mjs");
});

addBenchmark("path", "path.isAbsolute('/usr/bin/yaso')", () => {
    path.isAbsolute("/usr/bin/yaso");
});

startBenchmarking();

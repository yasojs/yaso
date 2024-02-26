import process from "process";
import { addBenchmark, startBenchmarking } from "../benchmarker.mjs";

addBenchmark("process", "process.cwd()", process.cwd);
addBenchmark("process", "process.hrtime()", process.hrtime);
addBenchmark("process", "process.hrtime.bigint()", process.hrtime.bigint);

startBenchmarking();

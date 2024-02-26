import os from "os";
import { addBenchmark, startBenchmarking } from "../benchmarker.mjs";

addBenchmark("os", "os.arch()", os.arch);
addBenchmark("os", "os.machine()", os.machine);
addBenchmark("os", "os.type()", os.type);
addBenchmark("os", "os.version()", os.version);
addBenchmark("os", "os.tmpdir()", os.tmpdir);

startBenchmarking();

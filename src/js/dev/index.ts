import { workerURL } from "./worker";

new Worker(workerURL, { type: "module" });

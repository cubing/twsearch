// TODO: remove once `esbuild` supports relative URLs: https://github.com/evanw/esbuild/pull/2508
export const workerURL = import.meta.url;

if (globalThis.WorkerGlobalScope) {
  import("./test");
}

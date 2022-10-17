import { barelyServe } from "barely-a-dev-server";

barelyServe({
	entryRoot: "./src/js",
	outDir: "./.cache/dev",
	esbuildOptions: {
		external: ["fs", "path"],
		minify: false
	},
});


// TODO:
// - Hack `fs`, `path`
// - remove module["exports"]
// - change rel. path calculation

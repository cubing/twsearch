#!/usr/bin/env node

const newargs = [];

// Work around https://github.com/rust-lang/rust/issues/60059#issuecomment-1553616479

for (const argument of process.argv.slice(2)) {
	if (argument.startsWith('-Wl')) {
		const newwlargs = [];
		for (const wlarg of argument.split(',').slice(1)) {
			if (wlarg.startsWith('-plugin-opt')) {
				console.log(`clang-wrapper-apple.js: Ignoring linker argument ${wlarg}`);
			} else {
				newwlargs.push(wlarg);
			}
		}
		if (newwlargs.length > 0) {
			newargs.push('-Wl,' + newwlargs.join(','));
		}
	} else {
		newargs.push(argument);
	}
}

console.log(newargs)

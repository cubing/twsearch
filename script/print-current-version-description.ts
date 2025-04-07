#!/usr/bin/env bun

import { getCurrentVersionDescription } from "./lib/getCurrentVersionDescription";

console.log(await getCurrentVersionDescription());

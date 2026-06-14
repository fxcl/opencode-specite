#!/usr/bin/env node
"use strict";

const { existsSync } = require("fs");
const { join } = require("path");
const { spawn } = require("child_process");

const ext = process.platform === "win32" ? ".exe" : "";
const binaryPath = join(__dirname, "specite" + ext);

if (!existsSync(binaryPath)) {
  process.stderr.write(
    "specite: binary not found.\n" +
      "Try running `npm rebuild specite` to re-download it.\n",
  );
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: true,
});

child.on("error", (err) => {
  process.stderr.write("specite: " + err.message + "\n");
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});

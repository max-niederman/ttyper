#!/usr/bin/env node

import { spawn } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

// ESM doesn't support __dirname and __filename by default
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const packageJsonPath = path.join(__dirname, "package.json");
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
const { name, binDir } = packageJson;

const commandArgs = process.argv.slice(2);
const exeName = ["win32", "cygwin"].includes(process.platform)
  ? `${name}.exe`
  : name;
const cwd = path.join(__dirname, binDir);
const binPath = path.join(cwd, exeName);

const child = spawn(binPath, commandArgs, { stdio: "inherit", cwd });
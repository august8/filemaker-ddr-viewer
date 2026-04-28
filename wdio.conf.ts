import { fileURLToPath } from "url";
import path from "path";
import type { Options } from "@wdio/types";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

const BINARY = path.resolve(
  __dirname,
  "src-tauri/target/debug/filemaker-ddr-viewer.exe"
);

export const config: Options.Testrunner = {
  runner: "local",
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      project: "./tsconfig.json",
      transpileOnly: true,
    },
  },

  specs: ["./tests/e2e/**/*.spec.ts"],

  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: BINARY,
      },
    },
  ],

  services: [["tauri", {}]],
  hostname: "localhost",
  port: 4444,

  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },

  reporters: ["spec"],

  logLevel: "warn",
  bail: 1,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 3,
};

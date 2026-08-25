/* global console, document, process, window */

import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright-core";
import { createServer } from "vite";

import { allRunsPass, evaluateRun, SPIKE_CONFIG } from "./benchmark.ts";
import { parseOutputArgument } from "./runner-args.ts";

const spikeRoot = dirname(fileURLToPath(import.meta.url));

async function main() {
  const output = resolve(parseOutputArgument(process.argv.slice(2)));
  let server;
  let browser;
  try {
    server = await createServer({
      root: spikeRoot,
      logLevel: "warn",
      server: { host: "127.0.0.1", port: 0, strictPort: false },
    });
    await server.listen();
    const url = server.resolvedUrls?.local[0];
    if (!url) {
      throw new Error("Vite did not expose a loopback URL");
    }

    browser = await chromium.launch({ channel: "msedge", headless: false });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    const runtimeErrors = [];
    page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));
    page.on("console", (message) => {
      if (message.type() === "error") {
        runtimeErrors.push(`console: ${message.text()}`);
      }
    });
    await page.goto(url, { waitUntil: "networkidle" });
    await page.bringToFront();
    await page.waitForFunction(() => document.documentElement.dataset.spikeReady === "true");

    const runs = [];
    let errorCursor = 0;
    for (let run = 1; run <= 3; run += 1) {
      await page.bringToFront();
      const metrics = await page.evaluate(async (runNumber) => {
        const controller = window.__virtualScrollSpike;
        if (!controller?.ready) {
          throw new Error("benchmark controller is not ready");
        }
        return controller.run(runNumber);
      }, run);
      await page.waitForTimeout(500);
      metrics.errors.push(...runtimeErrors.slice(errorCursor));
      errorCursor = runtimeErrors.length;
      runs.push(metrics);
    }

    const config = SPIKE_CONFIG;
    const verdicts = runs.map((metrics) => ({ run: metrics.run, ...evaluateRun(metrics) }));
    const result = {
      capturedAt: new Date().toISOString(),
      environment: {
        browser: await browser.version(),
        node: process.version,
        viewport: { width: 1440, height: 1000 },
        visible: true,
      },
      command: `pnpm run spike:virtual-scroll -- --output ${output}`,
      config,
      runs,
      verdicts,
      passed: allRunsPass(runs),
    };
    await mkdir(dirname(output), { recursive: true });
    await writeFile(output, `${JSON.stringify(result, null, 2)}\n`, "utf8");
    console.log(JSON.stringify(result, null, 2));
    if (!result.passed) {
      process.exitCode = 1;
    }
  } finally {
    await browser?.close();
    await server?.close();
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack ?? error.message : String(error));
  process.exitCode = 1;
});

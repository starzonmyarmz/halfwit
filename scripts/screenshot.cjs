const path = require("node:path");
const { webkit } = require("playwright");

const [out, width, height, version] = process.argv.slice(2);
const ui = path.join(__dirname, "..", "ui");

(async () => {
  const browser = await webkit.launch();
  const context = await browser.newContext({
    viewport: { width: Number(width), height: Number(height) },
    deviceScaleFactor: 2,
    reducedMotion: "reduce",
  });
  const page = await context.newPage();
  await page.route("http://ui.test/**", (route) => {
    const file = new URL(route.request().url()).pathname.slice(1) || "index.html";
    return route.fulfill({ path: path.join(ui, file) });
  });
  await page.goto(`http://ui.test/?version=${encodeURIComponent(version)}`);
  await page.waitForLoadState("networkidle");
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(250);
  await page.screenshot({ path: out });
  await browser.close();
})().catch((error) => {
  console.error(error);
  process.exit(1);
});

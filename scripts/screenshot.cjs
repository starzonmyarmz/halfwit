const path = require("node:path");
const { webkit } = require("playwright");

const [out, width, height, version] = process.argv.slice(2);
const ui = path.join(__dirname, "..", "ui");

// Room around the window for the object's drop shadow, which falls
// mostly downward.
const margin = { top: 48, side: 48, bottom: 80 };
const backdrop = `
  html {
    box-sizing: border-box !important;
    padding: ${margin.top}px ${margin.side}px ${margin.bottom}px !important;
  }
  html, body {
    background: transparent !important;
    overflow: visible !important;
  }
`;

(async () => {
  const browser = await webkit.launch();
  const context = await browser.newContext({
    viewport: { width: Number(width) + 2 * margin.side, height: Number(height) + margin.top + margin.bottom },
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
  await page.addStyleTag({ content: backdrop });
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(250);
  await page.screenshot({ path: out, omitBackground: true });
  await browser.close();
})().catch((error) => {
  console.error(error);
  process.exit(1);
});

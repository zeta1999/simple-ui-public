import { chromium } from 'playwright';

// Capture a full-page screenshot of the running desktop/React app.
// Usage: node screenshot.mjs [outputPath] [url]
const outputPath = process.argv[2] || 'screenshot.png';
const url = process.argv[3] || 'http://localhost:5173';

(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();

    console.log(`Navigating to ${url} ...`);
    try {
        await page.goto(url, { waitUntil: 'networkidle' });
        // Wait an extra second for any animations or data loads.
        await page.waitForTimeout(1000);

        await page.screenshot({ path: outputPath, fullPage: true });
        console.log(`Screenshot saved to ${outputPath}`);
    } catch (e) {
        console.error(e);
        process.exit(1);
    }

    await browser.close();
})();

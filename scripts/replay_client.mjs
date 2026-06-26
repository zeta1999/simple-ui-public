import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';

const scenarioPath = process.argv[2];
if (!scenarioPath) {
    console.error("Usage: node replay_client.mjs <scenario.json>");
    process.exit(1);
}

const scenario = JSON.parse(fs.readFileSync(scenarioPath, 'utf8'));

async function describeGUI(page) {
    // Generate a semantic description of the GUI
    const description = await page.evaluate(() => {
        let desc = [];
        
        // Extract headers
        document.querySelectorAll('h1, h2, h3').forEach(h => {
            desc.push(`Heading: ${h.innerText.trim()}`);
        });

        // Extract radios and inputs (for questions/forms)
        document.querySelectorAll('input[type="radio"]').forEach((r) => {
            const label = r.closest('label');
            const labelText = label ? label.innerText.trim() : r.value;
            desc.push(`Radio [${r.checked ? 'x' : ' '}]: ${labelText} (name: ${r.name})`);
        });

        // Extract spreadsheets/tables
        document.querySelectorAll('table').forEach(t => {
            desc.push(`Table:`);
            t.querySelectorAll('tr').forEach(tr => {
                let row = [];
                tr.querySelectorAll('td, th').forEach(td => {
                    let input = td.querySelector('input');
                    if (input) row.push(`[Input: ${input.value}]`);
                    else row.push(td.innerText.trim());
                });
                desc.push(`  | ${row.join(' | ')} |`);
            });
        });

        // Extract buttons
        document.querySelectorAll('button').forEach(b => {
            desc.push(`Button: ${b.innerText.trim()}`);
        });
        
        return desc.join('\n');
    });
    
    console.log("=== GUI STATE ===");
    console.log(description || "Empty GUI");
    console.log("=================\n");
}

(async () => {
    console.log("Launching headless browser...");
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    
    // We navigate to the built Vite output or running dev server.
    // For this test, we assume the dev server is running on 5173.
    // Alternatively, we could serve the `dist` folder.
    const url = 'http://localhost:5173';
    console.log(`Navigating to ${url} ...`);
    try {
        await page.goto(url, { waitUntil: 'networkidle' });
    } catch (e) {
        console.error(`Failed to navigate to ${url}. Make sure the React app is running (e.g., 'cd graphical && npm run dev').`);
        process.exit(1);
    }
    
    console.log("Initial GUI State:");
    await describeGUI(page);
    
    for (const event of scenario) {
        console.log(`\nExecuting event: ${JSON.stringify(event)}`);
        if (event.action === 'click') {
            await page.click(event.target);
        } else if (event.action === 'type') {
            await page.fill(event.target, event.value);
            await page.keyboard.press('Enter'); // Trigger recalculation if needed
        }
        
        // Wait a bit for UI/WASM to update
        await page.waitForTimeout(500);
        await describeGUI(page);
    }
    
    await browser.close();
    console.log("Replay finished.");
})();

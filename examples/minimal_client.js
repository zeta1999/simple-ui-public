// examples/minimal_client.js
//
// This is a minimal NodeJS client demonstrating how an external program
// communicates with the Extended Markdown IPC daemon.
// It listens to UI events (Server-Sent Events) and pushes Markdown updates.

const http = require('http');

const IPC_PORT = 3030; // Assuming the Rust IPC daemon runs on 3030
const SEC_TOKEN = "super_secret_token_123";

// 1. Listen for UI Events via SSE (Server-Sent Events)
function listenForEvents() {
    console.log("Connecting to IPC Event Stream...");
    
    const options = {
        hostname: '127.0.0.1',
        port: IPC_PORT,
        path: '/events',
        method: 'GET',
        headers: {
            'x-sec-token': SEC_TOKEN,
            'Accept': 'text/event-stream'
        }
    };

    const req = http.request(options, (res) => {
        if (res.statusCode === 401) {
            console.error("Unauthorized: Invalid security token.");
            return;
        }

        res.setEncoding('utf8');
        res.on('data', (chunk) => {
            // SSE chunks look like: `data: {"event_type":"input","element_id":"q_deployment"...}\n\n`
            if (chunk.startsWith('data: ')) {
                const jsonStr = chunk.replace('data: ', '').trim();
                if (jsonStr) {
                    try {
                        const event = JSON.parse(jsonStr);
                        console.log("\n[IPC EVENT RECEIVED] 🔔");
                        console.log(`Type: ${event.event_type}`);
                        console.log(`Element: ${event.element_id}`);
                        console.log(`Payload:`, event.payload);
                        
                        // Example: If user clicked something, update the UI!
                        if (event.event_type === 'input' && event.element_id === 'q_deployment') {
                            updateMarkdown("Thank you for choosing: " + event.payload.value);
                        }
                    } catch(e) {
                        // ignore keepalives
                    }
                }
            }
        });
    });

    req.on('error', (e) => {
        console.error(`Connection error (Is the Rust daemon running?): ${e.message}`);
    });

    req.end();
}

// 2. Push a Markdown Update to the TUI/GUI
function updateMarkdown(newText) {
    console.log("\n[IPC PUSH] Sending dynamic markdown update...");
    
    const markdownContent = `# Live Update\n\n${newText}\n\n\`\`\`plot\n{"type":"bar"}\n\`\`\``;
    
    const options = {
        hostname: '127.0.0.1',
        port: IPC_PORT,
        path: '/update',
        method: 'POST',
        headers: {
            'x-sec-token': SEC_TOKEN,
            'Content-Type': 'text/plain',
            'Content-Length': Buffer.byteLength(markdownContent)
        }
    };

    const req = http.request(options, (res) => {
        res.on('data', (chunk) => {
            console.log(`[IPC PUSH SUCCESS] Response: ${chunk}`);
        });
    });

    req.write(markdownContent);
    req.end();
}

// Start listening
listenForEvents();

// Simulate pushing an update 2 seconds after connection
setTimeout(() => {
    updateMarkdown("This content was injected by the minimal NodeJS client over HTTP!");
}, 2000);

import init, { resize_width_wasm, resize_height_wasm } from './seam_stitching.js';

let wasmInitialized = false;

async function initWasm() {
    await init();
    wasmInitialized = true;
    console.log("Wasm initialized in worker");
}

initWasm().catch(err => {
    console.error("Wasm Init Failed:", err);
    self.postMessage({ type: 'error', error: 'Wasm Init Failed: ' + err.toString() });
});

self.onmessage = async (e) => {
    console.log("Worker received message:", e.data);
    
    if (!wasmInitialized) {
        console.log("Wasm not yet initialized, waiting...");
        // Simple polling for initialization
        while (!wasmInitialized) {
            await new Promise(resolve => setTimeout(resolve, 10));
        }
    }

    const { type, data, width, height, target } = e.data;
    
    const progressHandler = (p) => {
        console.log("Progress update:", p);
        self.postMessage({ type: 'progress', progress: p });
    };

    try {
        let result;
        if (type === 'resize_width') {
            console.log("Starting resize_width...");
            result = resize_width_wasm(data, width, height, target, progressHandler);
        } else if (type === 'resize_height') {
            console.log("Starting resize_height...");
            result = resize_height_wasm(data, width, height, target, progressHandler);
        } else {
            throw new Error(`Unknown resize type: ${type}`);
        }
        
        console.log("Resize complete, sending result back");
        self.postMessage({ type: 'done', data: result });
    } catch (err) {
        console.error("Worker Error:", err);
        self.postMessage({ type: 'error', error: err.toString() });
    }
};

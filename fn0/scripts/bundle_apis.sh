#!/bin/bash
# Bundle deno API JS files for fn0 onejs using deno bundle

set -e

OUTPUT_DIR="js"
OUTPUT_FILE="$OUTPUT_DIR/bundled_apis.js"

# Create entry point that imports from ext: modules (deno understands these)
cat > /tmp/fn0_api_entry.js << 'EOF'
// Import WinterCG APIs from deno's ext: modules
import * as console from "ext:deno_console/01_console.js";
import * as headers from "ext:deno_fetch/20_headers.js";
import * as request from "ext:deno_fetch/23_request.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as fetch from "ext:deno_fetch/26_fetch.js";
import * as encoding from "ext:deno_web/08_text_encoding.js";
import * as timers from "ext:deno_web/02_timers.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as event from "ext:deno_web/02_event.js";
import * as abortSignal from "ext:deno_web/03_abort_signal.js";
import * as url from "ext:deno_url/00_url.js";
import * as crypto from "ext:deno_crypto/00_crypto.js";
import { DOMException } from "ext:deno_web/01_dom_exception.js";

// Export everything to globalThis (like bootstrap.js does)
const { ObjectDefineProperties } = globalThis.__primordials;

ObjectDefineProperties(globalThis, {
  console: { value: console.console, writable: true, enumerable: true, configurable: true },
  Request: { value: request.Request, writable: true, enumerable: true, configurable: true },
  Response: { value: response.Response, writable: true, enumerable: true, configurable: true },
  Headers: { value: headers.Headers, writable: true, enumerable: true, configurable: true },
  fetch: { value: fetch.fetch, writable: true, enumerable: true, configurable: true },
  TextDecoder: { value: encoding.TextDecoder, writable: true, enumerable: true, configurable: true },
  TextEncoder: { value: encoding.TextEncoder, writable: true, enumerable: true, configurable: true },
  setTimeout: { value: timers.setTimeout, writable: true, enumerable: true, configurable: true },
  clearTimeout: { value: timers.clearTimeout, writable: true, enumerable: true, configurable: true },
  setInterval: { value: timers.setInterval, writable: true, enumerable: true, configurable: true },
  clearInterval: { value: timers.clearInterval, writable: true, enumerable: true, configurable: true },
  ReadableStream: { value: streams.ReadableStream, writable: true, enumerable: true, configurable: true },
  WritableStream: { value: streams.WritableStream, writable: true, enumerable: true, configurable: true },
  TransformStream: { value: streams.TransformStream, writable: true, enumerable: true, configurable: true },
  Event: { value: event.Event, writable: true, enumerable: true, configurable: true },
  EventTarget: { value: event.EventTarget, writable: true, enumerable: true, configurable: true },
  CustomEvent: { value: event.CustomEvent, writable: true, enumerable: true, configurable: true },
  AbortController: { value: abortSignal.AbortController, writable: true, enumerable: true, configurable: true },
  AbortSignal: { value: abortSignal.AbortSignal, writable: true, enumerable: true, configurable: true },
  URL: { value: url.URL, writable: true, enumerable: true, configurable: true },
  URLSearchParams: { value: url.URLSearchParams, writable: true, enumerable: true, configurable: true },
  Crypto: { value: crypto.Crypto, writable: true, enumerable: true, configurable: true },
  crypto: { value: crypto.crypto, writable: true, enumerable: true, configurable: true },
  CryptoKey: { value: crypto.CryptoKey, writable: true, enumerable: true, configurable: true },
  SubtleCrypto: { value: crypto.SubtleCrypto, writable: true, enumerable: true, configurable: true },
  DOMException: { value: DOMException, writable: true, enumerable: true, configurable: true },
});

console.log("fn0 onejs: WinterCG APIs initialized");
EOF

echo "Bundling with deno..."
deno bundle /tmp/fn0_api_entry.js "$OUTPUT_FILE"

# Clean up
rm -f /tmp/fn0_api_entry.js

echo "✅ Bundled APIs saved to: $OUTPUT_FILE"
wc -l "$OUTPUT_FILE"

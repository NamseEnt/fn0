// Minimal bootstrap for fn0 onejs runtime
import * as console from "ext:deno_console/01_console.js";
import * as fetch from "ext:deno_fetch/26_fetch.js";
import * as headers from "ext:deno_fetch/20_headers.js";
import * as request from "ext:deno_fetch/23_request.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as encoding from "ext:deno_web/08_text_encoding.js";
import * as timers from "ext:deno_web/02_timers.js";
import * as url from "ext:deno_url/00_url.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as event from "ext:deno_web/02_event.js";
import * as abortSignal from "ext:deno_web/03_abort_signal.js";
import * as crypto from "ext:deno_crypto/00_crypto.js";
import { DOMException } from "ext:deno_web/01_dom_exception.js";

const { ObjectDefineProperty, ObjectDefineProperties } = globalThis.__primordials;

// Set up console first
delete globalThis.console;
ObjectDefineProperties(globalThis, {
  console: {
    value: new console.Console((msg, level) => globalThis.core.print(msg, level > 1)),
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Fetch API
  Request: {
    value: request.Request,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  Response: {
    value: response.Response,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  Headers: {
    value: headers.Headers,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  fetch: {
    value: fetch.fetch,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Encoding
  TextDecoder: {
    value: encoding.TextDecoder,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  TextEncoder: {
    value: encoding.TextEncoder,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // URL
  URL: {
    value: url.URL,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  URLSearchParams: {
    value: url.URLSearchParams,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Timers
  setTimeout: {
    value: timers.setTimeout,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  clearTimeout: {
    value: timers.clearTimeout,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  setInterval: {
    value: timers.setInterval,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  clearInterval: {
    value: timers.clearInterval,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Streams
  ReadableStream: {
    value: streams.ReadableStream,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  WritableStream: {
    value: streams.WritableStream,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  TransformStream: {
    value: streams.TransformStream,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Events
  Event: {
    value: event.Event,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  EventTarget: {
    value: event.EventTarget,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  CustomEvent: {
    value: event.CustomEvent,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // AbortController
  AbortController: {
    value: abortSignal.AbortController,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  AbortSignal: {
    value: abortSignal.AbortSignal,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Crypto
  Crypto: {
    value: crypto.Crypto,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  crypto: {
    value: crypto.crypto,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  CryptoKey: {
    value: crypto.CryptoKey,
    configurable: true,
    enumerable: true,
    writable: true,
  },
  SubtleCrypto: {
    value: crypto.SubtleCrypto,
    configurable: true,
    enumerable: true,
    writable: true,
  },

  // Error types
  DOMException: {
    value: DOMException,
    configurable: true,
    enumerable: true,
    writable: true,
  },
});

// Store user's handle function when module is loaded
globalThis.__fn0_user_handle = null;

// Hook for module loading - will be called from Rust
globalThis.__fn0_set_handler = (handler) => {
  if (typeof handler !== "function") {
    throw new TypeError("exported handle must be a function");
  }
  globalThis.__fn0_user_handle = handler;
};

// Helper to validate module exports
globalThis.__fn0_validate_module = (module) => {
  if (typeof module.handle === "function") {
    globalThis.__fn0_set_handler(module.handle);
  } else {
    throw new TypeError('Module must export a named "handle" function');
  }
};

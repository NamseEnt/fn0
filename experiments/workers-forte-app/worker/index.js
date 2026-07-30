// Imported before the heavy modules so its evaluation timestamp brackets theirs;
// ES imports are hoisted, so a `Date.now()` at the top of this file would not.
import { workerLoadStartedAt } from './load-clock.js';
import { instantiate } from './gen/backend.js';
import backendCore from './gen/backend.core.wasm';
import backendCore2 from './gen/backend.core2.wasm';
import backendCore3 from './gen/backend.core3.wasm';
import {
  P3Fields,
  P3Request,
  guestStreamToReadableStream,
  makeImports,
} from '../../workers-jspi-probe/worker/shim.js';
import '../dist/server.js';

const CORE_MODULES = {
  'backend.core.wasm': backendCore,
  'backend.core2.wasm': backendCore2,
  'backend.core3.wasm': backendCore3,
};

const NEXT_HEADER = 'x-fn0-next';

const moduleLoadMillis = Date.now() - workerLoadStartedAt;
let isolateRequestCount = 0;

// fn0 lets the SSR pass reach the guest over loopback HTTP; a Worker has no
// loopback server, so `server.js` is pointed at an origin that never leaves the
// isolate and `fetch` dispatches it straight into a component instance. Cookies
// and body travel in the request itself, so this holds no per-request state and
// is safe when the isolate interleaves requests.
const SELF_INVOKE_ORIGIN = 'http://forte-self-invoke';
globalThis.__FORTE_SELF_INVOKE_BASE__ = SELF_INVOKE_ORIGIN;

const nativeFetch = globalThis.fetch.bind(globalThis);
globalThis.fetch = async (input, init) => {
  const request = new Request(input, init);
  if (!request.url.startsWith(`${SELF_INVOKE_ORIGIN}/`)) {
    return nativeFetch(input, init);
  }
  return callGuest(request, selfInvokeEnv);
};

let selfInvokeEnv = {};

// The probe found that a component instance serves exactly one `handle` call, so
// every call gets a fresh one — see ../../workers-jspi-probe/README.md.
function instantiateComponent(env) {
  return instantiate(
    (path) => CORE_MODULES[path],
    makeImports({ environment: Object.entries(env).filter(([, v]) => typeof v === 'string') }),
    (module, imports) => new WebAssembly.Instance(module, imports),
  );
}

function toGuestRequest(request) {
  const url = new URL(request.url);
  const [guestRequest] = P3Request.new(
    P3Fields.fromHeaders(request.headers),
    undefined,
    Promise.resolve({ tag: 'ok', val: undefined }),
    undefined,
  );
  guestRequest.setMethod({ tag: request.method.toLowerCase() });
  guestRequest.setScheme({ tag: url.protocol === 'http:' ? 'HTTP' : 'HTTPS' });
  guestRequest.setAuthority(url.host);
  guestRequest.setPathWithQuery(`${url.pathname}${url.search}`);
  guestRequest.nativeBody = request.body;
  return guestRequest;
}

async function delegateToJs(request, wasmHeaders, propsJson) {
  const jsHeaders = new Headers(request.headers);
  jsHeaders.set('content-type', 'application/json');
  const jsResponse = await globalThis.handler(
    new Request(request.url, { method: 'POST', headers: jsHeaders, body: propsJson }),
  );

  const merged = new Headers(jsResponse.headers);
  for (const cookie of wasmHeaders.getSetCookie?.() ?? []) {
    merged.append('set-cookie', cookie);
  }
  return new Response(jsResponse.body, { status: jsResponse.status, headers: merged });
}

async function callGuest(request, env) {
  const root = instantiateComponent(env);
  let guestResponse;
  try {
    guestResponse = await root.handler.handle(toGuestRequest(request));
  } catch (error) {
    return new Response(`guest handle() failed: ${describe(error)}\n`, {
      status: 500,
      headers: { 'content-type': 'text/plain' },
    });
  }
  const headers = guestResponse.headers?.toHeaders() ?? new Headers();
  const body =
    guestResponse.contents === undefined
      ? null
      : guestStreamToReadableStream(guestResponse.contents);
  return new Response(body, { status: guestResponse.getStatusCode(), headers });
}

export default {
  async fetch(request, env) {
    selfInvokeEnv = env;

    // Asset-first routing 404s any path that is not a file, so the Worker runs
    // first and asks the asset router itself. Only for GET/HEAD: the asset
    // router reads the body, which would leave nothing for the guest.
    if (request.method === 'GET' || request.method === 'HEAD') {
      const asset = await env.ASSETS.fetch(request);
      if (asset.status !== 404) {
        return asset;
      }
    }

    // Bisects where the CPU goes when a request trips the plan's CPU limit.
    const stage = new URL(request.url).searchParams.get('stage');
    isolateRequestCount += 1;
    if (stage === 'instantiate' || stage === 'none') {
      const startedAt = Date.now();
      if (stage === 'instantiate') {
        instantiateComponent(env);
      }
      return new Response(
        `stage=${stage} instantiate=${Date.now() - startedAt}ms` +
          ` moduleLoad=${moduleLoadMillis}ms isolateRequest=${isolateRequestCount}\n`,
      );
    }

    const wasmStartedAt = Date.now();
    const wasmResponse = await callGuest(request, env);
    const wasmMillis = Date.now() - wasmStartedAt;

    if (stage === 'wasm') {
      wasmResponse.headers.set('x-fn0-wasm-ms', String(wasmMillis));
      return wasmResponse;
    }

    if (wasmResponse.status !== 200 || wasmResponse.headers.get(NEXT_HEADER) !== 'js') {
      wasmResponse.headers.set('x-fn0-wasm-ms', String(wasmMillis));
      return wasmResponse;
    }

    const propsJson = await wasmResponse.text();
    const ssrStartedAt = Date.now();
    const response = await delegateToJs(request, wasmResponse.headers, propsJson);
    response.headers.set('x-fn0-wasm-ms', String(wasmMillis));
    response.headers.set('x-fn0-ssr-ms', String(Date.now() - ssrStartedAt));
    return response;
  },
};

function describe(error) {
  if (error instanceof Error) {
    return `${error.stack ?? error.message}`;
  }
  return JSON.stringify(error);
}

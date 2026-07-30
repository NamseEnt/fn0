import { instantiate } from './gen/probe.js';
import probeCore from './gen/probe.core.wasm';
import probeCore2 from './gen/probe.core2.wasm';
import probeCore3 from './gen/probe.core3.wasm';
import { P3Fields, P3Request, guestStreamToReadableStream, makeImports } from './shim.js';

const CORE_MODULES = {
  'probe.core.wasm': probeCore,
  'probe.core2.wasm': probeCore2,
  'probe.core3.wasm': probeCore3,
};

let reusedInstance;

// A component instance serves exactly one `handle` call: reusing it makes every
// request after the first hang forever with no error, so the default is a fresh
// instance per request. `?reuse=1` reproduces the hang — see README.md.
function componentInstance({ reuse }) {
  if (!reuse) {
    return { root: instantiateComponent(), instantiateMillis: null };
  }
  if (reusedInstance === undefined) {
    reusedInstance = instantiateComponent();
  }
  return { root: reusedInstance, instantiateMillis: null };
}

function instantiateComponent() {
  return instantiate(
    (path) => CORE_MODULES[path],
    makeImports(),
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
  return guestRequest;
}

// Burns CPU without touching the clock: Workers freezes timers during synchronous
// execution, so a Date.now() spin loop would never exit. Used to find where the
// plan's CPU limit actually bites — see README.md.
function burnCpu(iterations) {
  let accumulator = 0;
  for (let iteration = 0; iteration < iterations; iteration += 1) {
    accumulator += Math.sqrt(iteration % 10_000);
  }
  return accumulator;
}

export default {
  async fetch(request) {
    const searchParams = new URL(request.url).searchParams;

    const burnIterations = Number(searchParams.get('burn') ?? 0);
    if (burnIterations > 0) {
      const accumulator = burnCpu(burnIterations);
      return new Response(`burned ${burnIterations} iterations, accumulator ${accumulator}\n`, {
        headers: { 'content-type': 'text/plain' },
      });
    }

    const reuse = searchParams.get('reuse') === '1';

    const instantiateStartedAt = Date.now();
    const { root } = componentInstance({ reuse });
    const instantiateMillis = Date.now() - instantiateStartedAt;

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
    headers.set('x-probe-instantiate-ms', String(instantiateMillis));
    headers.set('x-probe-instance-reused', String(reuse));
    const body =
      guestResponse.contents === undefined
        ? null
        : guestStreamToReadableStream(guestResponse.contents);

    return new Response(body, { status: guestResponse.getStatusCode(), headers });
  },
};

function describe(error) {
  if (error instanceof Error) {
    return `${error.stack ?? error.message}`;
  }
  return JSON.stringify(error);
}

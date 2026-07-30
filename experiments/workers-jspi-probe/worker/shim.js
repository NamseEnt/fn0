// Workers-side host implementation of the imports jco's transpiled glue asks for.
// The full surface is enumerated by `grep -o "fnName = '[^']*'" gen/probe.js`; nothing
// beyond that list is implemented, because anything more would be untested guesswork.
//
// Two jco conventions drive the shapes here:
//   - a WIT `result<T, E>` is a returned T or a thrown E
//   - a WIT `future<T>` is a Promise, and a WIT `stream<u8>` is a ReadableStream
//     (host -> guest) or a jco `Stream` object with `read({count})` (guest -> host)

const READ_CHUNK_ELEMENTS = 65536;
const NANOS_PER_MILLI = 1_000_000n;

// Workers rejects these on an outbound fetch; it sets them itself.
const UNSETTABLE_REQUEST_HEADERS = new Set([
  'host',
  'connection',
  'content-length',
  'transfer-encoding',
  'upgrade',
  'keep-alive',
]);

const textDecoder = new TextDecoder();
const textEncoder = new TextEncoder();

class Pollable {
  ready() {
    return true;
  }

  block() {}
}

class IoError {
  toDebugString() {
    return 'io-error';
  }
}

class InputStream {}

class OutputStream {
  constructor(label) {
    this.label = label ?? 'stdout';
  }

  checkWrite() {
    return BigInt(READ_CHUNK_ELEMENTS);
  }

  write(bytes) {
    const text = textDecoder.decode(new Uint8Array(bytes)).replace(/\n$/, '');
    if (text.length > 0) {
      console.log(`[guest ${this.label}] ${text}`);
    }
  }

  blockingFlush() {}

  subscribe() {
    return new Pollable();
  }
}

export class P3Fields {
  static fromList(entries) {
    const fields = new P3Fields();
    fields.entries = entries.map(([name, value]) => [name, new Uint8Array(value)]);
    return fields;
  }

  static fromHeaders(headers) {
    return P3Fields.fromList([...headers].map(([name, value]) => [name, textEncoder.encode(value)]));
  }

  toHeaders({ skipUnsettable = false } = {}) {
    const headers = new Headers();
    for (const [name, value] of this.entries ?? []) {
      if (skipUnsettable && UNSETTABLE_REQUEST_HEADERS.has(name.toLowerCase())) {
        continue;
      }
      headers.append(name, textDecoder.decode(value));
    }
    return headers;
  }
}

export class P3RequestOptions {}

export class P3Request {
  static new(headers, contents, trailers, options) {
    const request = new P3Request();
    request.headers = headers;
    request.contents = contents;
    request.trailers = trailers;
    request.options = options;
    request.method = { tag: 'get' };
    request.scheme = undefined;
    request.authority = undefined;
    request.pathWithQuery = undefined;
    return [request, Promise.resolve({ tag: 'ok', val: undefined })];
  }

  setMethod(method) {
    this.method = method;
  }

  setPathWithQuery(pathWithQuery) {
    this.pathWithQuery = pathWithQuery;
  }

  setScheme(scheme) {
    this.scheme = scheme;
  }

  setAuthority(authority) {
    this.authority = authority;
  }
}

export class P3Response {
  static new(headers, contents, trailers) {
    const response = new P3Response();
    response.headers = headers;
    response.contents = contents;
    response.trailers = trailers;
    response.statusCode = 200;
    return [response, Promise.resolve({ tag: 'ok', val: undefined })];
  }

  static consumeBody(response, _result) {
    const body = response.nativeBody ?? emptyReadableStream();
    return [body, Promise.resolve({ tag: 'ok', val: undefined })];
  }

  getStatusCode() {
    return this.statusCode ?? 200;
  }

  setStatusCode(statusCode) {
    this.statusCode = statusCode;
  }
}

function emptyReadableStream() {
  return new ReadableStream({
    start(controller) {
      controller.close();
    },
  });
}

// A stream the guest wrote (jco `Stream`) read out as a web ReadableStream.
export function guestStreamToReadableStream(stream) {
  return new ReadableStream({
    async pull(controller) {
      const { value, done } = await stream.read({ count: READ_CHUNK_ELEMENTS });
      if (value !== undefined && value.length > 0) {
        controller.enqueue(value instanceof Uint8Array ? value : new Uint8Array(value));
      }
      if (done) {
        controller.close();
      }
    },
    cancel() {
      stream[Symbol.dispose]?.();
    },
  });
}

function methodToString(method) {
  if (method === undefined) {
    return 'GET';
  }
  return method.tag === 'other' ? method.val : method.tag.toUpperCase();
}

function schemeToString(scheme) {
  if (scheme === undefined) {
    return 'https';
  }
  switch (scheme.tag) {
    case 'HTTP':
      return 'http';
    case 'HTTPS':
      return 'https';
    default:
      return scheme.val;
  }
}

function internalError(message) {
  return { tag: 'internal-error', val: message };
}

async function send(request) {
  if (request.authority === undefined || request.authority === '') {
    throw internalError('outbound request has no authority');
  }
  const url = `${schemeToString(request.scheme)}://${request.authority}${request.pathWithQuery ?? '/'}`;
  const method = methodToString(request.method);

  const init = {
    method,
    headers: request.headers?.toHeaders({ skipUnsettable: true }) ?? new Headers(),
  };
  if (request.contents !== undefined && method !== 'GET' && method !== 'HEAD') {
    init.body = guestStreamToReadableStream(request.contents);
  }

  let nativeResponse;
  try {
    nativeResponse = await fetch(url, init);
  } catch (error) {
    throw internalError(`fetch(${url}) failed: ${error}`);
  }

  const response = new P3Response();
  response.statusCode = nativeResponse.status;
  response.headers = P3Fields.fromHeaders(nativeResponse.headers);
  response.nativeBody = nativeResponse.body;
  return response;
}

export function makeImports() {
  const stdout = new OutputStream('stdout');
  const stderr = new OutputStream('stderr');
  const stdin = new InputStream();

  return {
    'wasi:cli/environment': { getEnvironment: () => [] },
    'wasi:cli/exit': {
      exit: (status) => {
        throw new Error(`guest called exit(${JSON.stringify(status)})`);
      },
    },
    'wasi:cli/stdin': { getStdin: () => stdin },
    'wasi:cli/stdout': { getStdout: () => stdout },
    'wasi:cli/stderr': { getStderr: () => stderr },
    'wasi:cli/terminal-input': { TerminalInput: class TerminalInput {} },
    'wasi:cli/terminal-output': { TerminalOutput: class TerminalOutput {} },
    'wasi:cli/terminal-stdin': { getTerminalStdin: () => undefined },
    'wasi:cli/terminal-stdout': { getTerminalStdout: () => undefined },
    'wasi:cli/terminal-stderr': { getTerminalStderr: () => undefined },
    'wasi:clocks/types': {},
    'wasi:clocks/monotonic-clock': {
      now: () => BigInt(Date.now()) * NANOS_PER_MILLI,
      waitFor: (duration) =>
        new Promise((resolve) => setTimeout(resolve, Number(duration / NANOS_PER_MILLI))),
      subscribeDuration: () => new Pollable(),
    },
    'wasi:io/error': { Error: IoError },
    'wasi:io/poll': { Pollable, poll: () => [] },
    'wasi:io/streams': { InputStream, OutputStream },
    'wasi:http/types': {
      Fields: P3Fields,
      Request: P3Request,
      RequestOptions: P3RequestOptions,
      Response: P3Response,
    },
    'wasi:http/client': { send },
  };
}

export interface Env {
  SIGNY_URL?: string;
  SIGNY_ACCESS_CLIENT_ID?: string;
  SIGNY_ACCESS_CLIENT_SECRET?: string;
  PLATFORM_TELEMETRY_TENANT?: string;
  CANARY_URL?: string;
  CANARY_ACCESS_CLIENT_ID?: string;
  CANARY_ACCESS_CLIENT_SECRET?: string;
}

export interface AccessCredential {
  clientId: string;
  clientSecret: string;
}

export interface ConsoleConfig {
  signyUrl: string;
  signyAccess: AccessCredential;
  platformTelemetryTenant: string;
  canaryUrl: string;
  canaryAccess: AccessCredential;
}

export class MissingBindingError extends Error {
  readonly binding: string;

  constructor(binding: string) {
    super(`missing Worker binding ${binding}`);
    this.binding = binding;
  }
}

function required(env: Env, name: keyof Env): string {
  const value = env[name];
  if (value === undefined || value === "") {
    throw new MissingBindingError(name);
  }
  return value;
}

export function readConfig(env: Env): ConsoleConfig {
  return {
    signyUrl: required(env, "SIGNY_URL").replace(/\/+$/, ""),
    signyAccess: {
      clientId: required(env, "SIGNY_ACCESS_CLIENT_ID"),
      clientSecret: required(env, "SIGNY_ACCESS_CLIENT_SECRET"),
    },
    platformTelemetryTenant: required(env, "PLATFORM_TELEMETRY_TENANT"),
    canaryUrl: required(env, "CANARY_URL").replace(/\/+$/, ""),
    canaryAccess: {
      clientId: required(env, "CANARY_ACCESS_CLIENT_ID"),
      clientSecret: required(env, "CANARY_ACCESS_CLIENT_SECRET"),
    },
  };
}

export function accessHeaders(credential: AccessCredential): Record<string, string> {
  return {
    "CF-Access-Client-Id": credential.clientId,
    "CF-Access-Client-Secret": credential.clientSecret,
  };
}

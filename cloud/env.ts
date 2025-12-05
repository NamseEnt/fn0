export function envCloudflareApiToken() {
  return makeSureEnv("CLOUDFLARE_API_TOKEN");
}

export function envCloudflareAccountId() {
  return makeSureEnv("CLOUDFLARE_ACCOUNT_ID");
}

export function envDomain() {
  return makeSureEnv("DOMAIN");
}

export function envAwsWatchdogRegion() {
  return makeSureEnv("AWS_WATCHDOG_REGION");
}

export function envOciComputeWorkerRegion() {
  return makeSureEnv("OCI_COMPUTE_WORKER_REGION");
}

function makeSureEnv(name: string) {
  const envVar = process.env[name];
  if (!envVar) {
    throw new Error(`Environment variable ${name} is not set`);
  }
  return envVar;
}

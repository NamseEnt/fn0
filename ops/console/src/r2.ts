import type { ConsoleConfig } from "./config.ts";
import type { Dependencies } from "./runtime.ts";

const GRAPHQL_ENDPOINT = "https://api.cloudflare.com/client/v4/graphql";
const WINDOW_SECONDS = 24 * 60 * 60;
const MAX_GROUPS = 10000;
const QUERY = `query Fn0R2Analytics($accountTag: string!, $startDate: Time!, $endDate: Time!) {
  viewer {
    accounts(filter: { accountTag: $accountTag }) {
      storage: r2StorageAdaptiveGroups(
        limit: 10000
        filter: { datetime_geq: $startDate, datetime_leq: $endDate }
        orderBy: [datetime_DESC]
      ) {
        max { payloadSize metadataSize objectCount }
        dimensions { bucketName datetime }
      }
      operations: r2OperationsAdaptiveGroups(
        limit: 10000
        filter: { datetime_geq: $startDate, datetime_leq: $endDate }
      ) {
        sum { requests }
        dimensions { bucketName actionStatus datetime }
      }
      bandwidth: r2BandwidthUsageAdaptiveGroups(
        limit: 10000
        filter: { datetime_geq: $startDate, datetime_lt: $endDate }
      ) {
        sum { bytesUpload bytesDownload }
        dimensions { bucketName datetimeFiveMinutes }
      }
    }
  }
}`;

export interface R2AnalyticsBucket {
  bucket_name: string;
  payload_bytes: number | null;
  metadata_bytes: number | null;
  object_count: number | null;
  operation_count: number | null;
  success_operations: number | null;
  user_error_operations: number | null;
  internal_error_operations: number | null;
  upload_bytes: number | null;
  download_bytes: number | null;
  analytics_freshness_seconds: number | null;
}

export interface R2AnalyticsResponse {
  status: "ok" | "unavailable";
  window_seconds: number;
  fetched_at: string;
  buckets: R2AnalyticsBucket[];
}

interface StorageGroup {
  max?: { payloadSize?: unknown; metadataSize?: unknown; objectCount?: unknown };
  dimensions?: { bucketName?: unknown; datetime?: unknown };
}

interface OperationsGroup {
  sum?: { requests?: unknown };
  dimensions?: { bucketName?: unknown; actionStatus?: unknown; datetime?: unknown };
}

interface BandwidthGroup {
  sum?: { bytesUpload?: unknown; bytesDownload?: unknown };
  dimensions?: { bucketName?: unknown; datetimeFiveMinutes?: unknown };
}

interface GraphqlResponse {
  data?: {
    viewer?: {
      accounts?: Array<{
        storage?: StorageGroup[];
        operations?: OperationsGroup[];
        bandwidth?: BandwidthGroup[];
      }>;
    };
  };
  errors?: unknown[];
}

interface BucketAccumulator {
  bucket_name: string;
  payload_bytes: number | null;
  metadata_bytes: number | null;
  object_count: number | null;
  operation_count: number;
  success_operations: number;
  user_error_operations: number;
  internal_error_operations: number;
  upload_bytes: number;
  download_bytes: number;
  newest_sample_ms: number | null;
  storage_datetime_ms: number;
}

function numeric(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string" && value.trim() !== "") {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

function timestamp(value: unknown): number | null {
  if (typeof value !== "string") return null;
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function bucketName(value: unknown): string | null {
  return typeof value === "string" && value.startsWith("fn0-") ? value : null;
}

function accumulator(buckets: Map<string, BucketAccumulator>, name: string): BucketAccumulator {
  const existing = buckets.get(name);
  if (existing) return existing;
  const created: BucketAccumulator = {
    bucket_name: name,
    payload_bytes: null,
    metadata_bytes: null,
    object_count: null,
    operation_count: 0,
    success_operations: 0,
    user_error_operations: 0,
    internal_error_operations: 0,
    upload_bytes: 0,
    download_bytes: 0,
    newest_sample_ms: null,
    storage_datetime_ms: 0,
  };
  buckets.set(name, created);
  return created;
}

function recordNewestSample(bucket: BucketAccumulator, value: unknown): void {
  const sampledAt = timestamp(value);
  if (sampledAt !== null) {
    bucket.newest_sample_ms = Math.max(bucket.newest_sample_ms ?? 0, sampledAt);
  }
}

function emptyResponse(status: R2AnalyticsResponse["status"], nowMs: number): R2AnalyticsResponse {
  return {
    status,
    window_seconds: WINDOW_SECONDS,
    fetched_at: new Date(nowMs).toISOString(),
    buckets: [],
  };
}

function validConfig(config: ConsoleConfig): config is ConsoleConfig & {
  cloudflareAccountId: string;
  cloudflareAnalyticsApiToken: string;
} {
  return config.cloudflareAccountId !== null && config.cloudflareAnalyticsApiToken !== null;
}

export async function r2Analytics(dependencies: Dependencies): Promise<R2AnalyticsResponse> {
  const nowMs = dependencies.nowMs();
  const { config } = dependencies;
  if (!validConfig(config)) return emptyResponse("unavailable", nowMs);
  const startDate = new Date(nowMs - WINDOW_SECONDS * 1000).toISOString();
  const endDate = new Date(nowMs).toISOString();
  let response: Response;
  try {
    response = await dependencies.fetch(GRAPHQL_ENDPOINT, {
      method: "POST",
      headers: {
        authorization: `Bearer ${config.cloudflareAnalyticsApiToken}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        query: QUERY,
        variables: { accountTag: config.cloudflareAccountId, startDate, endDate },
      }),
      signal: AbortSignal.timeout(dependencies.deadlines.signyMs),
    });
  } catch {
    return emptyResponse("unavailable", nowMs);
  }
  if (!response.ok) return emptyResponse("unavailable", nowMs);
  let payload: GraphqlResponse;
  try {
    payload = await response.json() as GraphqlResponse;
  } catch {
    return emptyResponse("unavailable", nowMs);
  }
  if (payload.errors?.length) return emptyResponse("unavailable", nowMs);
  const account = payload.data?.viewer?.accounts?.[0];
  if (
    !account ||
    !Array.isArray(account.storage) ||
    !Array.isArray(account.operations) ||
    !Array.isArray(account.bandwidth) ||
    account.storage.length >= MAX_GROUPS ||
    account.operations.length >= MAX_GROUPS ||
    account.bandwidth.length >= MAX_GROUPS
  ) {
    return emptyResponse("unavailable", nowMs);
  }
  const buckets = new Map<string, BucketAccumulator>();

  for (const group of account.storage ?? []) {
    const name = bucketName(group.dimensions?.bucketName);
    if (!name) continue;
    const bucket = accumulator(buckets, name);
    const sampleTime = timestamp(group.dimensions?.datetime);
    if (sampleTime !== null && sampleTime >= bucket.storage_datetime_ms) {
      bucket.storage_datetime_ms = sampleTime;
      bucket.payload_bytes = numeric(group.max?.payloadSize);
      bucket.metadata_bytes = numeric(group.max?.metadataSize);
      bucket.object_count = numeric(group.max?.objectCount);
    }
    recordNewestSample(bucket, group.dimensions?.datetime);
  }

  for (const group of account.operations ?? []) {
    const name = bucketName(group.dimensions?.bucketName);
    if (!name) continue;
    const bucket = accumulator(buckets, name);
    const count = numeric(group.sum?.requests);
    if (count !== null) {
      bucket.operation_count += count;
      switch (group.dimensions?.actionStatus) {
        case "success": bucket.success_operations += count; break;
        case "userError": bucket.user_error_operations += count; break;
        case "internalError": bucket.internal_error_operations += count; break;
      }
    }
    recordNewestSample(bucket, group.dimensions?.datetime);
  }

  for (const group of account.bandwidth ?? []) {
    const name = bucketName(group.dimensions?.bucketName);
    if (!name) continue;
    const bucket = accumulator(buckets, name);
    bucket.upload_bytes += numeric(group.sum?.bytesUpload) ?? 0;
    bucket.download_bytes += numeric(group.sum?.bytesDownload) ?? 0;
    recordNewestSample(bucket, group.dimensions?.datetimeFiveMinutes);
  }

  const result = [...buckets.values()]
    .sort((left, right) => left.bucket_name.localeCompare(right.bucket_name))
    .map((bucket): R2AnalyticsBucket => ({
      bucket_name: bucket.bucket_name,
      payload_bytes: bucket.payload_bytes,
      metadata_bytes: bucket.metadata_bytes,
      object_count: bucket.object_count,
      operation_count: bucket.operation_count,
      success_operations: bucket.success_operations,
      user_error_operations: bucket.user_error_operations,
      internal_error_operations: bucket.internal_error_operations,
      upload_bytes: bucket.upload_bytes,
      download_bytes: bucket.download_bytes,
      analytics_freshness_seconds: bucket.newest_sample_ms === null
        ? null
        : Math.max(0, (nowMs - bucket.newest_sample_ms) / 1000),
    }));
  return {
    status: "ok",
    window_seconds: WINDOW_SECONDS,
    fetched_at: new Date(nowMs).toISOString(),
    buckets: result,
  };
}

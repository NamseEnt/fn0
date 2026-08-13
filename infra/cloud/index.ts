import * as fn0 from "@pulumi/fn0";
import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as oci from "@pulumi/oci";
import * as cloudflare from "@pulumi/cloudflare";
import * as grafana from "@pulumiverse/grafana";
import * as random from "@pulumi/random";
import * as crypto from "node:crypto";

const config = new pulumi.Config();

const accountId = config.require("cloudflareAccountId");
const zoneId = config.require("cloudflareZoneId");
const domain = config.require("domain");

const suffix = new fn0.Suffix("suffix").result;

const envEncryptionKey = new random.RandomBytes("fn0-env-encryption-key", {
  length: 32,
});

const adminSigningKey = new random.RandomBytes("fn0-admin-signing-key", {
  length: 32,
});

const dns = new fn0.CloudflareDns("cloudflare-dns", {
  suffix,
  accountId,
  zoneId,
  domain,
});

new cloudflare.ZoneSetting("ssl-mode", {
  zoneId,
  settingId: "ssl",
  value: "strict",
});

new cloudflare.Ruleset("native-static-page-cache", {
  zoneId,
  name: "native-static-page-cache",
  kind: "zone",
  phase: "http_request_cache_settings",
  rules: [
    {
      action: "set_cache_settings",
      actionParameters: {
        cache: true,
        edgeTtl: {
          mode: "bypass_by_default",
        },
        // Cache purge cannot reach browsers, so the origin's `no-cache` must
        // survive to the client or a deploy purge leaves repeat visitors on
        // the previous version until their browser TTL expires.
        browserTtl: {
          mode: "respect_origin",
        },
      },
      // `PURGE` is not client traffic: a Cache Rule that does not match during
      // a single-file purge makes that purge silently no-op (it still returns
      // `success: true`). Dropping it breaks by-URL invalidation with no error.
      // https://developers.cloudflare.com/cache/how-to/purge-cache/purge-by-single-file/
      expression:
        'http.request.method in {"GET" "HEAD" "PURGE"} and not starts_with(http.request.uri.path, "/__fn0_queue_task/")',
    },
  ],
});

// Cache misses in any colo fill from an upper-tier colo instead of the origin,
// so a miss costs an R2 Class B operation once per upper tier rather than once
// per edge location. Smart Topology is available on the Free plan.
// https://developers.cloudflare.com/cache/how-to/tiered-cache/
new cloudflare.TieredCache("smart-tiered-cache", {
  zoneId,
  value: "on",
});

const forteDb = new fn0.ForteDb(
  "forte-db",
  {
    organizationSlug: config.require("tursoOrganizationSlug"),
    location: config.require("tursoLocation"),
  },
  {},
);

const tursoApiToken = new pulumi.Config("turso").requireSecret("apiToken");

new fn0.ControlProjectBootstrap(
  "control-project-bootstrap",
  {
    organizationSlug: config.require("tursoOrganizationSlug"),
    groupName: forteDb.groupName,
    projectId: "fn0-control",
  },
  { dependsOn: [forteDb] },
);

const forteR2 = new fn0.ForteR2(
  "forte-r2",
  {
    accountId,
    zoneId,
    domain,
    staticHostname: `forte-static.${domain}`,
    bucketName: pulumi.interpolate`fn0-forte-static-${suffix}`,
  },
  {},
);

const cwasmCompilerRegion = "ap-northeast-1";

const cwasmCompilerBucketR = new aws.s3.Bucket("cwasm-compiler-bucket", {
  region: cwasmCompilerRegion,
});

const cwasmCompilerEcrR = new aws.ecr.Repository("cwasm-compiler-ecr", {
  region: cwasmCompilerRegion,
  imageTagMutability: "MUTABLE",
  forceDelete: true,
});

new aws.ecr.RepositoryPolicy("cwasm-compiler-ecr-policy", {
  region: cwasmCompilerRegion,
  repository: cwasmCompilerEcrR.name,
  policy: aws.getCallerIdentityOutput({}).accountId.apply((accountId) =>
    JSON.stringify({
      Version: "2008-10-17",
      Statement: [
        {
          Sid: "LambdaECRImageRetrievalPolicy",
          Effect: "Allow",
          Principal: { Service: "lambda.amazonaws.com" },
          Action: ["ecr:BatchGetImage", "ecr:GetDownloadUrlForLayer"],
          Condition: {
            StringLike: {
              "aws:sourceArn": `arn:aws:lambda:${cwasmCompilerRegion}:${accountId}:function:*`,
            },
          },
        },
      ],
    }),
  ),
});

const cwasmCompilerRoleR = new aws.iam.Role("cwasm-compiler-role", {
  assumeRolePolicy: JSON.stringify({
    Version: "2012-10-17",
    Statement: [
      {
        Effect: "Allow",
        Principal: { Service: "lambda.amazonaws.com" },
        Action: "sts:AssumeRole",
      },
    ],
  }),
  managedPolicyArns: [aws.iam.ManagedPolicy.AWSLambdaBasicExecutionRole],
});

new aws.iam.RolePolicy("cwasm-compiler-role-policy", {
  role: cwasmCompilerRoleR.name,
  policy: cwasmCompilerBucketR.arn.apply((arn) =>
    JSON.stringify({
      Version: "2012-10-17",
      Statement: [
        {
          Effect: "Allow",
          Action: ["s3:ListBucket"],
          Resource: arn,
        },
        {
          Effect: "Allow",
          Action: ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"],
          Resource: `${arn}/*`,
        },
      ],
    }),
  ),
});

const awsCallerIdentity = aws.getCallerIdentityOutput({});

const controlAwsUser = new aws.iam.User("control-aws-user", {
  name: "fn0-control",
});

const controlAwsAccessKey = new aws.iam.AccessKey("control-aws-access-key", {
  user: controlAwsUser.name,
});

const cwasmCompilerBuilderUser = new aws.iam.User(
  "cwasm-compiler-builder-user",
  {
    name: "fn0-cwasm-compiler-builder",
  },
);

const cwasmCompilerBuilderAccessKey = new aws.iam.AccessKey(
  "cwasm-compiler-builder-access-key",
  { user: cwasmCompilerBuilderUser.name },
);

new aws.iam.UserPolicy("cwasm-compiler-builder-user-policy", {
  user: cwasmCompilerBuilderUser.name,
  policy: pulumi
    .all([
      cwasmCompilerEcrR.arn,
      cwasmCompilerRoleR.arn,
      awsCallerIdentity.accountId,
    ])
    .apply(([ecrArn, roleArn, accountId]) =>
      JSON.stringify({
        Version: "2012-10-17",
        Statement: [
          {
            Effect: "Allow",
            Action: "ecr:GetAuthorizationToken",
            Resource: "*",
          },
          {
            Effect: "Allow",
            Action: [
              "ecr:BatchCheckLayerAvailability",
              "ecr:BatchDeleteImage",
              "ecr:BatchGetImage",
              "ecr:CompleteLayerUpload",
              "ecr:DescribeImages",
              "ecr:GetDownloadUrlForLayer",
              "ecr:InitiateLayerUpload",
              "ecr:PutImage",
              "ecr:UploadLayerPart",
            ],
            Resource: ecrArn,
          },
          {
            Effect: "Allow",
            Action: [
              "lambda:CreateFunction",
              "lambda:GetFunction",
              "lambda:GetFunctionConfiguration",
              "lambda:UpdateFunctionCode",
              "lambda:UpdateFunctionConfiguration",
              "lambda:DeleteFunction",
            ],
            Resource: `arn:aws:lambda:${cwasmCompilerRegion}:${accountId}:function:fn0-cwasm-compiler-*`,
          },
          {
            Effect: "Allow",
            Action: "iam:PassRole",
            Resource: roleArn,
            Condition: {
              StringEquals: { "iam:PassedToService": "lambda.amazonaws.com" },
            },
          },
        ],
      }),
    ),
});

new aws.iam.UserPolicy("control-aws-user-policy", {
  user: controlAwsUser.name,
  policy: pulumi
    .all([cwasmCompilerBucketR.arn, awsCallerIdentity.accountId])
    .apply(([bucketArn, accountId]) =>
      JSON.stringify({
        Version: "2012-10-17",
        Statement: [
          {
            Effect: "Allow",
            Action: ["s3:ListBucket"],
            Resource: bucketArn,
          },
          {
            Effect: "Allow",
            Action: ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"],
            Resource: `${bucketArn}/*`,
          },
          {
            Effect: "Allow",
            Action: ["lambda:InvokeFunction"],
            Resource: `arn:aws:lambda:${cwasmCompilerRegion}:${accountId}:function:fn0-cwasm-compiler-*`,
          },
        ],
      }),
    ),
});

const ociGlobalVault = new fn0.OciGlobalVault("oci-global-vault", {
  region: config.require("ociVaultRegion"),
  allowedSubdomain: "fn0-control",
});

const controlDek = new oci.kms.GeneratedKey(
  "control-dek",
  {
    cryptoEndpoint: ociGlobalVault.cryptoEndpoint,
    keyId: ociGlobalVault.keyOcid,
    keyShape: { algorithm: "AES", length: 32 },
    includePlaintextKey: true,
  },
  { dependsOn: [ociGlobalVault] },
);

const fn0TokenHmacKey = new random.RandomBytes("fn0-token-hmac-key", {
  length: 32,
});

const controlCookieSecret = new random.RandomBytes(
  "fn0-control-cookie-secret",
  {
    length: 32,
  },
);

const controlAdminToken = new random.RandomBytes("fn0-control-admin-token", {
  length: 32,
});

const websocketBearer = new random.RandomPassword("websocket-bearer", {
  length: 48,
  special: false,
});

const bundleStoreR2 = new fn0.BundleStoreR2(
  "bundle-store-r2",
  {
    accountId,
    bucketName: pulumi.interpolate`fn0-bundle-store-${suffix}`,
  },
  {},
);

const metricsBackupR2 = new fn0.MetricsBackupR2(
  "metrics-backup-r2",
  {
    accountId,
    bucketName: pulumi.interpolate`fn0-metrics-backup-${suffix}`,
  },
  {},
);

// The metrics node's basic-auth credential is minted here rather than on the
// node because every worker's Alloy has to present it, so it is a shared
// credential either way. Keeping it in the stack also means rebuilding the
// node (fresh machine + vmrestore) does not rotate it, so the workers stay
// untouched.
const metricsHostname = config.require("metricsHostname");
const metricsBasicAuthUsernameValue = "fn0";
const metricsBasicAuthSecret = new random.RandomBytes(
  "fn0-metrics-basic-auth-password",
  { length: 24 },
);

const bundleStoreR2Worker = new fn0.BundleStoreR2Worker(
  "bundle-store-r2-worker",
  {
    accountId,
    scriptName: pulumi.interpolate`fn0-bundle-store-r2-worker-${suffix}`,
    bucketName: bundleStoreR2.bucketName,
    queueId: bundleStoreR2.queueId,
    controlUrl: pulumi.interpolate`https://${domain}`,
    adminToken: controlAdminToken.base64,
  },
  {},
);

const githubClientId = config.require("githubClientId");
const githubClientSecret = config.requireSecret("githubClientSecret");

function aesGcmEncryptToBase64(dekBase64: string, plaintext: string): string {
  const key = Buffer.from(dekBase64, "base64");
  if (key.length !== 32) {
    throw new Error(`DEK must be 32 bytes, got ${key.length}`);
  }
  const nonce = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv("aes-256-gcm", key, nonce);
  const ct = Buffer.concat([cipher.update(plaintext, "utf8"), cipher.final()]);
  const tag = cipher.getAuthTag();
  return Buffer.concat([nonce, ct, tag]).toString("base64");
}

const controlGithubSecretCt = pulumi
  .all([controlDek.plaintext, githubClientSecret])
  .apply(([dek, secret]) => aesGcmEncryptToBase64(dek, secret));

const controlTokenHmacCt = pulumi
  .all([controlDek.plaintext, fn0TokenHmacKey.base64])
  .apply(([dek, hmac]) => aesGcmEncryptToBase64(dek, hmac));

const controlCookieSecretCt = pulumi
  .all([controlDek.plaintext, controlCookieSecret.base64])
  .apply(([dek, secret]) => aesGcmEncryptToBase64(dek, secret));

const controlAdminTokenCt = pulumi
  .all([controlDek.plaintext, controlAdminToken.base64])
  .apply(([dek, token]) => aesGcmEncryptToBase64(dek, token));

const controlWebsocketBearerCt = pulumi
  .all([controlDek.plaintext, websocketBearer.result])
  .apply(([dek, token]) => aesGcmEncryptToBase64(dek, token));

const bundleStoreR2AccessKeyIdCt = pulumi
  .all([controlDek.plaintext, bundleStoreR2.accessKeyId])
  .apply(([dek, value]) => aesGcmEncryptToBase64(dek, value));

const bundleStoreR2SecretAccessKeyCt = pulumi
  .all([controlDek.plaintext, bundleStoreR2.secretAccessKey])
  .apply(([dek, value]) => aesGcmEncryptToBase64(dek, value));

const controlLambdaAccessKeyIdCt = pulumi
  .all([controlDek.plaintext, controlAwsAccessKey.id])
  .apply(([dek, value]) => aesGcmEncryptToBase64(dek, value));

const controlLambdaSecretAccessKeyCt = pulumi
  .all([controlDek.plaintext, controlAwsAccessKey.secret])
  .apply(([dek, value]) => aesGcmEncryptToBase64(dek, value));

const controlTursoApiTokenCt = pulumi
  .all([controlDek.plaintext, tursoApiToken])
  .apply(([dek, value]) => aesGcmEncryptToBase64(dek, value));

const controlEnvYamlBootstrap = pulumi
  .all([
    controlDek.ciphertext,
    githubClientId,
    controlGithubSecretCt,
    controlTokenHmacCt,
    controlCookieSecretCt,
    controlAdminTokenCt,
    controlWebsocketBearerCt,
    bundleStoreR2.accountId,
    bundleStoreR2.bucketName,
    bundleStoreR2AccessKeyIdCt,
    bundleStoreR2SecretAccessKeyCt,
    pulumi.output(cwasmCompilerRegion),
    controlLambdaAccessKeyIdCt,
    controlLambdaSecretAccessKeyCt,
    controlTursoApiTokenCt,
    pulumi.output(config.require("tursoOrganizationSlug")),
    forteDb.groupName,
  ])
  .apply(
    ([
      dekCt,
      clientId,
      ghCt,
      hmacCt,
      cookieCt,
      adminCt,
      websocketBearerCt,
      r2AccountId,
      r2Bucket,
      r2KeyCt,
      r2SecretCt,
      lambdaRegion,
      lambdaKeyCt,
      lambdaSecretCt,
      tursoApiTokenCt,
      tursoOrgSlug,
      tursoGroupName,
    ]) =>
      [
        "__dek:",
        `  encrypted: ${dekCt}`,
        `GITHUB_CLIENT_ID: ${clientId}`,
        "GITHUB_CLIENT_SECRET:",
        `  secret: ${ghCt}`,
        "FN0_TOKEN_HMAC_KEY:",
        `  secret: ${hmacCt}`,
        "COOKIE_SECRET:",
        `  secret: ${cookieCt}`,
        "FN0_CONTROL_ADMIN_TOKEN:",
        `  secret: ${adminCt}`,
        "FN0_WEBSOCKET_QUIC_BEARER:",
        `  secret: ${websocketBearerCt}`,
        `FN0_BUNDLE_STORE_ACCOUNT_ID: ${r2AccountId}`,
        `FN0_BUNDLE_STORE_BUCKET: ${r2Bucket}`,
        "FN0_BUNDLE_STORE_ACCESS_KEY_ID:",
        `  secret: ${r2KeyCt}`,
        "FN0_BUNDLE_STORE_SECRET_ACCESS_KEY:",
        `  secret: ${r2SecretCt}`,
        `FN0_LAMBDA_REGION: ${lambdaRegion}`,
        "FN0_LAMBDA_ACCESS_KEY_ID:",
        `  secret: ${lambdaKeyCt}`,
        "FN0_LAMBDA_SECRET_ACCESS_KEY:",
        `  secret: ${lambdaSecretCt}`,
        "FN0_TURSO_API_TOKEN:",
        `  secret: ${tursoApiTokenCt}`,
        `FN0_TURSO_ORG_SLUG: ${tursoOrgSlug}`,
        `FN0_TURSO_GROUP_NAME: ${tursoGroupName}`,
        "",
      ].join("\n"),
  );

const grafanaStack = grafana.cloud.getStackOutput({
  slug: config.require("grafanaSlug"),
});
const grafanaCloudAccessPolicyToken = new pulumi.Config(
  "grafana",
).requireSecret("cloudAccessPolicyToken");
const grafanaOtlpEndpoint = grafanaStack.otlpUrl.apply((url) => `${url}/otlp`);
const workerHostObservability = {
  prometheusUrl: grafanaStack.prometheusRemoteWriteEndpoint,
  prometheusUserId: grafanaStack.prometheusUserId.apply((id) => id.toString()),
  lokiUrl: grafanaStack.logsUrl,
  lokiUserId: grafanaStack.logsUserId.apply((id) => id.toString()),
  otlpUrl: grafanaOtlpEndpoint,
  otlpUserId: grafanaStack.id.apply((id) => id.toString()),
  basicAuthPassword: grafanaCloudAccessPolicyToken,
};

const ociFn0WorkerSite = new fn0.OciFn0WorkerSite("oci-fn0-worker-site", {
  region: config.require("ociComputeWorkerRegion"),
  count: 1,
  shape: "VM.Standard.A1.Flex",
  ocpus: 1,
  memoryInGbs: 6,
  websocketBearer: websocketBearer.result,
  workerAgentForteDb: {
    groupToken: forteDb.groupToken,
    hostSuffix: forteDb.hostSuffix,
  },
  worker: {
    tlsOrigin: {
      certPem: dns.certificate,
      keyPem: dns.privateKeyPem,
    },
    envEncryptionKeyBase64: envEncryptionKey.base64,
    forteDb: {
      groupToken: forteDb.groupToken,
      hostSuffix: forteDb.hostSuffix,
    },
    crossProjectEnqueueAllowedCallerProjectId: "fn0-control",
    crossProjectInvokeAllowedCallerProjectId: "fn0-control",
    vault: {
      cryptoEndpoint: ociGlobalVault.cryptoEndpoint,
      keyOcid: ociGlobalVault.keyOcid,
      region: ociGlobalVault.region,
      allowedSubdomain: ociGlobalVault.allowedSubdomain,
      workerCredentials: ociGlobalVault.workerCredentials,
    },
    hostObservability: workerHostObservability,
    bundleStorage: {
      bucketName: bundleStoreR2.bucketName,
      endpoint: bundleStoreR2.endpoint,
      region: "auto",
      accessKeyId: bundleStoreR2.accessKeyId,
      secretAccessKey: bundleStoreR2.secretAccessKey,
    },
    controlProjectId: "fn0-control",
    apex: {
      domain,
      projectId: "fn0-control",
    },
  },
});

// fn0 cloud assumes bring-your-own-Cloudflare: every project runs behind the
// owner's own zone, and that zone's DNS points here. This record is DNS-only
// (grey cloud) on purpose — it is a CNAME target, not a proxied edge, so the
// owner's zone is what terminates TLS/edge and the NLB IP stays a single
// place to update if it ever changes. Lives here (not inside CloudflareDns)
// because the NLB IP is an OciFn0WorkerSite output and a forward dependency
// would create a cycle.
new cloudflare.DnsRecord("oci-ap-osaka-1-nlb-a", {
  zoneId,
  name: pulumi.interpolate`oci-ap-osaka-1-nlb.${domain}`,
  type: "A",
  content: ociFn0WorkerSite.networkLoadBalancerPublicIp,
  ttl: 1,
  proxied: false,
});

// Apex fn0.dev → same worker NLB. The worker routes apex to the fn0-control
// project via FN0_APEX_DOMAIN/FN0_APEX_PROJECT_ID.
new cloudflare.DnsRecord("worker-apex-a", {
  zoneId,
  name: domain,
  type: "A",
  content: ociFn0WorkerSite.networkLoadBalancerPublicIp,
  ttl: 1,
  proxied: true,
});

// worker-lb.fn0.dev is the SaaS Custom Hostname fallback origin (see
// CloudflareDns.saasFallbackLbHostname). It needs its own explicit A record
// because the SaaS fallback record protection only attaches to hostnames
// with one (not any wildcard match), and there is no wildcard record left to
// match it implicitly anyway.
new cloudflare.DnsRecord("worker-lb-a", {
  zoneId,
  name: pulumi.interpolate`worker-lb.${domain}`,
  type: "A",
  content: ociFn0WorkerSite.networkLoadBalancerPublicIp,
  ttl: 1,
  proxied: true,
});

new fn0.EventBridgeCronTrigger("control-cron-trigger", {
  controlUrl: pulumi.interpolate`https://${domain}`,
  controlAdminToken: controlAdminToken.base64,
  awsRegion: cwasmCompilerRegion,
  suffix,
});

export const workerImageRegistries = pulumi.secret(
  ociFn0WorkerSite.workerImageRegistries,
);
export const cwasmBucket = ociFn0WorkerSite.cwasmBucket.bucketName;
export const s3Endpoint = ociFn0WorkerSite.cwasmBucket.endpoint;
export const s3Region = ociFn0WorkerSite.cwasmBucket.region;
export const s3AccessKeyId = pulumi.secret(
  ociFn0WorkerSite.cwasmBucket.accessKeyId,
);
export const s3SecretAccessKey = pulumi.secret(
  ociFn0WorkerSite.cwasmBucket.secretAccessKey,
);
export const workerSshPrivateKey = pulumi.secret(
  ociFn0WorkerSite.sshPrivateKey,
);
export const cwasmCompilerBucket = cwasmCompilerBucketR.bucket;
export const cwasmCompilerBucketRegion = cwasmCompilerRegion;
export const cwasmCompilerEcrRepository = cwasmCompilerEcrR.repositoryUrl;
export const cwasmCompilerRoleArn = cwasmCompilerRoleR.arn;
export const cwasmCompilerBuilderAccessKeyId = pulumi.secret(
  cwasmCompilerBuilderAccessKey.id,
);
export const cwasmCompilerBuilderSecretAccessKey = pulumi.secret(
  cwasmCompilerBuilderAccessKey.secret,
);
export const controlAwsAccessKeyId = pulumi.secret(controlAwsAccessKey.id);
export const controlAwsSecretAccessKey = pulumi.secret(
  controlAwsAccessKey.secret,
);
export const forteDbGroupToken = pulumi.secret(forteDb.groupToken);
export const forteDbHostSuffix = forteDb.hostSuffix;
export const controlDbUrl = pulumi.interpolate`https://fn0-control${forteDb.hostSuffix}`;
export const controlOwnerGithubId = config.requireNumber(
  "controlOwnerGithubId",
);
export const vaultCryptoEndpoint = ociGlobalVault.cryptoEndpoint;
export const vaultKeyOcid = ociGlobalVault.keyOcid;
// Appended here rather than inside the block above because the worker fleet
// is created after everything else control needs to know. Users CNAME their
// custom domain at this hostname when they attach it to a project on their
// own Cloudflare account; see the oci-ap-osaka-1-nlb DNS record above.
export const controlBootstrapEnvYaml = pulumi.secret(
  controlEnvYamlBootstrap.apply(
    (yaml) => `${yaml}FN0_WORKER_ORIGIN_HOSTNAME: oci-ap-osaka-1-nlb.${domain}\n`,
  ),
);
export const controlUrl = pulumi.interpolate`https://${domain}`;
export const controlAdminTokenBase64 = pulumi.secret(controlAdminToken.base64);
export const bundleStoreR2AccountId = bundleStoreR2.accountId;
export const bundleStoreR2BucketName = bundleStoreR2.bucketName;
export const bundleStoreR2Endpoint = bundleStoreR2.endpoint;
export const bundleStoreR2AccessKeyId = pulumi.secret(
  bundleStoreR2.accessKeyId,
);
export const bundleStoreR2SecretAccessKey = pulumi.secret(
  bundleStoreR2.secretAccessKey,
);
export const bundleStoreR2QueueId = bundleStoreR2.queueId;
export const bundleStoreR2WorkerScriptName = bundleStoreR2Worker.scriptName;
export const workerCompartmentId = ociFn0WorkerSite.compartmentId;
export const workerBastionId = ociFn0WorkerSite.bastionId;
// The operator's own Cloudflare account and zone. fn0-control is a connected
// project like any other, and `bootstrap-fn0-control.sh` provisions it here
// because it cannot call `forte cloudflare connect` against a control plane
// that is not serving yet.
export const apexDomain = domain;
export const cloudflareAccountId = accountId;
export const cloudflareZoneId = zoneId;
export const metricsWriteUrl = `https://${metricsHostname}/api/v1/write`;
export const metricsQueryUrl = `https://${metricsHostname}`;
export const metricsOtlpUrl = `https://${metricsHostname}/opentelemetry`;
export const metricsBasicAuthUsername = metricsBasicAuthUsernameValue;
export const metricsBasicAuthPassword = pulumi.secret(
  metricsBasicAuthSecret.base64,
);
export const metricsBackupR2BucketName = metricsBackupR2.bucketName;
export const metricsBackupR2Endpoint = metricsBackupR2.endpoint;
export const metricsBackupR2AccessKeyId = pulumi.secret(
  metricsBackupR2.accessKeyId,
);
export const metricsBackupR2SecretAccessKey = pulumi.secret(
  metricsBackupR2.secretAccessKey,
);

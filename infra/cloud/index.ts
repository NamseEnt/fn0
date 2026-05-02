import * as fn0 from "@pulumi/fn0";
import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as oci from "@pulumi/oci";
import * as cloudflare from "@pulumi/cloudflare";
import * as random from "@pulumi/random";

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

const docDb = new fn0.TursoDocDb("doc-db", {
  organizationSlug: config.require("tursoOrganizationSlug"),
  location: config.require("tursoLocation"),
});

const forteDb = new fn0.ForteDb(
  "forte-db",
  {
    organizationSlug: config.require("tursoOrganizationSlug"),
    location: config.require("tursoLocation"),
  },
  {}
);

const tursoApiToken = new pulumi.Config("turso").requireSecret("apiToken");

const forteR2 = new fn0.ForteR2(
  "forte-r2",
  {
    accountId,
    zoneId,
    domain,
    staticHostname: `forte-static.${domain}`,
    bucketName: pulumi.interpolate`fn0-forte-static-${suffix}`,
  },
  {}
);

const sccacheRegion = config.require("ociHeadQuarterRegion");

const sccacheCompartment = new oci.identity.Compartment("sccache-compartment", {
  name: pulumi.interpolate`fn0-sccache-${suffix}`,
  description: "Compartment for fn0 sccache S3-compatible bucket",
  enableDelete: true,
});

const sccacheNamespace = oci.objectstorage.getNamespaceOutput({}).namespace;

const sccacheBucket = new oci.objectstorage.Bucket("sccache-bucket", {
  compartmentId: sccacheCompartment.id,
  namespace: sccacheNamespace,
  name: pulumi.interpolate`fn0-sccache-${suffix}`,
});

const sccacheUser = new oci.identity.User("sccache-user", {
  name: pulumi.interpolate`fn0-sccache-${suffix}`,
  description: "User for fn0 sccache S3-compatible access",
});

const sccacheGroup = new oci.identity.Group("sccache-group", {
  name: pulumi.interpolate`fn0-sccache-group-${suffix}`,
  description: "Group for fn0 sccache bucket access",
});

new oci.identity.UserGroupMembership("sccache-membership", {
  userId: sccacheUser.id,
  groupId: sccacheGroup.id,
});

new oci.identity.Policy("sccache-policy", {
  compartmentId: sccacheCompartment.id,
  name: pulumi.interpolate`allow-sccache-${suffix}`,
  description: "Policy for sccache bucket access",
  statements: [
    pulumi.interpolate`Allow group ${sccacheGroup.name} to manage objects in compartment id ${sccacheCompartment.id}`,
    pulumi.interpolate`Allow group ${sccacheGroup.name} to manage buckets in compartment id ${sccacheCompartment.id}`,
  ],
});

const sccacheCustomerKey = new oci.identity.CustomerSecretKey(
  "sccache-customer-key",
  {
    userId: sccacheUser.id,
    displayName: "fn0-sccache-s3-compat",
  }
);

const sccacheEndpoint = pulumi.interpolate`https://${sccacheNamespace}.compat.objectstorage.${sccacheRegion}.oraclecloud.com`;

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
    })
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
    })
  ),
});

const awsCallerIdentity = aws.getCallerIdentityOutput({});

const hqAwsUser = new aws.iam.User("hq-aws-user", {
  name: "fn0-hq",
});

const hqAwsAccessKey = new aws.iam.AccessKey("hq-aws-access-key", {
  user: hqAwsUser.name,
});

const cwasmCompilerBuilderUser = new aws.iam.User("cwasm-compiler-builder-user", {
  name: "fn0-cwasm-compiler-builder",
});

const cwasmCompilerBuilderAccessKey = new aws.iam.AccessKey(
  "cwasm-compiler-builder-access-key",
  { user: cwasmCompilerBuilderUser.name }
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
      })
    ),
});

new aws.iam.UserPolicy("hq-aws-user-policy", {
  user: hqAwsUser.name,
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
      })
    ),
});

const ociHeadQuarterVcn = new fn0.OciHeadQuarterVcn("oci-head-quarter-vcn", {
  suffix,
  region: config.require("ociHeadQuarterRegion"),
});

const ociComputeWorker = new fn0.OciComputeWorker("oci-compute-worker", {
  region: config.require("ociComputeWorkerRegion"),
  hqIpv6CidrBlocks: ociHeadQuarterVcn.ipv6cidrBlocks,
});

const dnsProvider = {
  cloudflare: {
    zoneId,
    asteriskDomain: `*.${domain}`,
    apiToken: dns.dnsApiToken,
  },
};

const ociHeadQuarter = new fn0.OciHeadQuarter("oci-head-quarter", {
  suffix,
  ociRegion: config.require("ociHeadQuarterRegion"),
  compartmentId: ociHeadQuarterVcn.compartmentId,
  vcnId: ociHeadQuarterVcn.vcnId,
  ipv6cidrBlocks: ociHeadQuarterVcn.ipv6cidrBlocks,
  grafanaSlug: config.require("grafanaSlug"),
  grafanaRegion: config.require("grafanaRegion"),
  docDbUrl: docDb.url,
  docDbToken: docDb.token,
  forteDb: {
    apiToken: tursoApiToken,
    organizationSlug: config.require("tursoOrganizationSlug"),
    groupName: forteDb.groupName,
  },
  forteR2: {
    bucket: forteR2.bucketName,
    endpoint: forteR2.endpoint,
    accessKeyId: forteR2.accessKeyId,
    secretAccessKey: forteR2.secretAccessKey,
    publicBaseUrl: forteR2.publicBaseUrl,
  },
  awsRegion: cwasmCompilerRegion,
  wasmBucket: cwasmCompilerBucketR.bucket,
  cwasmBucket: {
    name: ociComputeWorker.cwasmBucket.bucketName,
    endpoint: ociComputeWorker.cwasmBucket.endpoint,
    region: ociComputeWorker.cwasmBucket.region,
    accessKeyId: ociComputeWorker.cwasmBucket.accessKeyId,
    secretAccessKey: ociComputeWorker.cwasmBucket.secretAccessKey,
  },
  awsAccessKeyId: hqAwsAccessKey.id,
  awsSecretAccessKey: hqAwsAccessKey.secret,
  envEncryptionKeyBase64: envEncryptionKey.base64,
  adminSigningKeyBase64: adminSigningKey.base64,
  sccacheBucket: sccacheBucket.name,
  sccacheRegion: sccacheRegion,
  sccacheEndpoint: sccacheEndpoint,
  sccacheAccessKeyId: sccacheCustomerKey.id,
  sccacheSecretAccessKey: sccacheCustomerKey.key,
  selfDnsHostname: `fn0-hq.${domain}`,
  selfDnsCloudflareZoneId: zoneId,
  selfDnsCloudflareApiToken: dns.dnsApiToken,
  dnsProvider,
  cloudflareSaas: {
    zoneId,
    apiToken: dns.saasApiToken,
  },
  sites: [
    {
      name: "oci-compute-vm",
      hostProvider: {
        ociComputeVm: {
          privateKeyBase64: ociComputeWorker.infraEnvs.OCI_PRIVATE_KEY_BASE64,
          userId: ociComputeWorker.infraEnvs.OCI_USER_ID,
          fingerprint: ociComputeWorker.infraEnvs.OCI_FINGERPRINT,
          tenancyId: ociComputeWorker.infraEnvs.OCI_TENANCY_ID,
          region: config.require("ociComputeWorkerRegion"),
          compartmentId: ociComputeWorker.compartmentId,
          availabilityDomain: ociComputeWorker.infraEnvs.OCI_AVAILABILITY_DOMAIN,
          shape: "VM.Standard.A1.Flex",
          ocpus: 1,
          physicsCpuCores: 1,
          memoryInGbs: 6,
          subnetId: ociComputeWorker.subnetId,
          imageId: ociComputeWorker.osImageId,
          envs: {
            CWASM_BUCKET: ociComputeWorker.cwasmBucket.bucketName,
            S3_ENDPOINT: ociComputeWorker.cwasmBucket.endpoint,
            S3_REGION: ociComputeWorker.cwasmBucket.region,
            AWS_ACCESS_KEY_ID: ociComputeWorker.cwasmBucket.accessKeyId,
            AWS_SECRET_ACCESS_KEY: ociComputeWorker.cwasmBucket.secretAccessKey,
            ORIGIN_CERT_PEM: dns.certificate,
            ORIGIN_KEY_PEM: dns.privateKeyPem,
            FN0_ENV_KEY_BASE64: envEncryptionKey.base64,
            FN0_ADMIN_SIGNING_KEY_BASE64: adminSigningKey.base64,
            TURSO_GROUP_TOKEN: forteDb.groupToken,
            TURSO_DB_HOST_SUFFIX: forteDb.hostSuffix,
          },
          sshAuthorizedKeys: ociComputeWorker.sshPublicKey,
          sshPrivateKeyBase64: ociComputeWorker.sshPrivateKey.apply(
            (key) => Buffer.from(key).toString("base64")
          ),
        },
      },
    },
  ],
});

export const kubeconfig = pulumi.secret(ociHeadQuarter.kubeconfig);
export const workerImageRegistries = pulumi.secret(ociComputeWorker.workerImageRegistries);
export const cwasmBucket = ociComputeWorker.cwasmBucket.bucketName;
export const s3Endpoint = ociComputeWorker.cwasmBucket.endpoint;
export const s3Region = ociComputeWorker.cwasmBucket.region;
export const s3AccessKeyId = pulumi.secret(ociComputeWorker.cwasmBucket.accessKeyId);
export const s3SecretAccessKey = pulumi.secret(ociComputeWorker.cwasmBucket.secretAccessKey);
export const workerSshPrivateKey = pulumi.secret(ociComputeWorker.sshPrivateKey);
export const sccacheBucketName = sccacheBucket.name;
export const sccacheBucketRegion = sccacheRegion;
export const sccacheBucketEndpoint = sccacheEndpoint;
export const sccacheAccessKeyId = pulumi.secret(sccacheCustomerKey.id);
export const sccacheSecretAccessKey = pulumi.secret(sccacheCustomerKey.key);
export const cwasmCompilerBucket = cwasmCompilerBucketR.bucket;
export const cwasmCompilerBucketRegion = cwasmCompilerRegion;
export const cwasmCompilerEcrRepository = cwasmCompilerEcrR.repositoryUrl;
export const cwasmCompilerRoleArn = cwasmCompilerRoleR.arn;
export const cwasmCompilerBuilderAccessKeyId = pulumi.secret(cwasmCompilerBuilderAccessKey.id);
export const cwasmCompilerBuilderSecretAccessKey = pulumi.secret(cwasmCompilerBuilderAccessKey.secret);
export const hqAwsAccessKeyId = pulumi.secret(hqAwsAccessKey.id);
export const hqAwsSecretAccessKey = pulumi.secret(hqAwsAccessKey.secret);
export const docDbUrl = docDb.url;
export const docDbToken = pulumi.secret(docDb.token);

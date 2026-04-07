import * as outputs from "../types/output";
/**
 * The access key for API operations. You can retrieve this
 * from the 'Security & Credentials' section of the AWS console.
 */
export declare const accessKey: string | undefined;
export declare const allowedAccountIds: string[] | undefined;
export declare const assumeRoleWithWebIdentity: outputs.config.AssumeRoleWithWebIdentity | undefined;
export declare const assumeRoles: outputs.config.AssumeRoles[] | undefined;
/**
 * File containing custom root and intermediate certificates. Can also be configured using the `AWS_CA_BUNDLE` environment variable. (Setting `caBundle` in the shared config file is not supported.)
 */
export declare const customCaBundle: string | undefined;
/**
 * Configuration block with settings to default resource tags across all resources.
 */
export declare const defaultTags: outputs.config.DefaultTags | undefined;
/**
 * Address of the EC2 metadata service endpoint to use. Can also be configured using the `AWS_EC2_METADATA_SERVICE_ENDPOINT` environment variable.
 */
export declare const ec2MetadataServiceEndpoint: string | undefined;
/**
 * Protocol to use with EC2 metadata service endpoint.Valid values are `IPv4` and `IPv6`. Can also be configured using the `AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE` environment variable.
 */
export declare const ec2MetadataServiceEndpointMode: string | undefined;
export declare const endpoints: outputs.config.Endpoints[] | undefined;
export declare const forbiddenAccountIds: string[] | undefined;
/**
 * URL of a proxy to use for HTTP requests when accessing the AWS API. Can also be set using the `HTTP_PROXY` or `httpProxy` environment variables.
 */
export declare const httpProxy: string | undefined;
/**
 * URL of a proxy to use for HTTPS requests when accessing the AWS API. Can also be set using the `HTTPS_PROXY` or `httpsProxy` environment variables.
 */
export declare const httpsProxy: string | undefined;
/**
 * Configuration block with settings to ignore resource tags across all resources.
 */
export declare const ignoreTags: outputs.config.IgnoreTags | undefined;
/**
 * Explicitly allow the provider to perform "insecure" SSL requests. If omitted, default value is `false`
 */
export declare const insecure: boolean | undefined;
/**
 * The maximum number of times an AWS API request is
 * being executed. If the API request still fails, an error is
 * thrown.
 */
export declare const maxRetries: number | undefined;
/**
 * Comma-separated list of hosts that should not use HTTP or HTTPS proxies. Can also be set using the `NO_PROXY` or `noProxy` environment variables.
 */
export declare const noProxy: string | undefined;
/**
 * The profile for API operations. If not set, the default profile
 * created with `aws configure` will be used.
 */
export declare const profile: string | undefined;
/**
 * The region where AWS operations will take place. Examples
 * are us-east-1, us-west-2, etc.
 */
export declare const region: string | undefined;
/**
 * Specifies how retries are attempted. Valid values are `standard` and `adaptive`. Can also be configured using the `AWS_RETRY_MODE` environment variable.
 */
export declare const retryMode: string | undefined;
/**
 * Specifies whether S3 API calls in the `us-east-1` region use the legacy global endpoint or a regional endpoint. Valid values are `legacy` or `regional`. Can also be configured using the `AWS_S3_US_EAST_1_REGIONAL_ENDPOINT` environment variable or the `s3UsEast1RegionalEndpoint` shared config file parameter
 */
export declare const s3UsEast1RegionalEndpoint: string | undefined;
/**
 * Set this to true to enable the request to use path-style addressing,
 * i.e., https://s3.amazonaws.com/BUCKET/KEY. By default, the S3 client will
 * use virtual hosted bucket addressing when possible
 * (https://BUCKET.s3.amazonaws.com/KEY). Specific to the Amazon S3 service.
 */
export declare const s3UsePathStyle: boolean | undefined;
/**
 * The secret key for API operations. You can retrieve this
 * from the 'Security & Credentials' section of the AWS console.
 */
export declare const secretKey: string | undefined;
/**
 * List of paths to shared config files. If not set, defaults to [~/.aws/config].
 */
export declare const sharedConfigFiles: string[] | undefined;
/**
 * List of paths to shared credentials files. If not set, defaults to [~/.aws/credentials].
 */
export declare const sharedCredentialsFiles: string[] | undefined;
/**
 * Skip the credentials validation via STS API. Used for AWS API implementations that do not have STS available/implemented.
 */
export declare const skipCredentialsValidation: boolean;
/**
 * Skip the AWS Metadata API check. Used for AWS API implementations that do not have a metadata api endpoint.
 */
export declare const skipMetadataApiCheck: boolean | undefined;
/**
 * Skip static validation of region name. Used by users of alternative AWS-like APIs or users w/ access to regions that are not public (yet).
 */
export declare const skipRegionValidation: boolean;
/**
 * Skip requesting the account ID. Used for AWS API implementations that do not have IAM/STS API and/or metadata API.
 */
export declare const skipRequestingAccountId: boolean | undefined;
/**
 * The region where AWS STS operations will take place. Examples
 * are us-east-1 and us-west-2.
 */
export declare const stsRegion: string | undefined;
/**
 * session token. A session token is only required if you are
 * using temporary security credentials.
 */
export declare const token: string | undefined;
/**
 * The capacity of the AWS SDK's token bucket rate limiter.
 */
export declare const tokenBucketRateLimiterCapacity: number | undefined;
/**
 * Resolve an endpoint with DualStack capability
 */
export declare const useDualstackEndpoint: boolean | undefined;
/**
 * Resolve an endpoint with FIPS capability
 */
export declare const useFipsEndpoint: boolean | undefined;

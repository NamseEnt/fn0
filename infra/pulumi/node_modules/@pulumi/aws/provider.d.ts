import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
/**
 * The provider type for the aws package. By default, resources use package-wide configuration
 * settings, however an explicit `Provider` instance may be created and passed during resource
 * construction to achieve fine-grained programmatic control over provider settings. See the
 * [documentation](https://www.pulumi.com/docs/reference/programming-model/#providers) for more information.
 */
export declare class Provider extends pulumi.ProviderResource {
    /**
     * Returns true if the given object is an instance of Provider.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Provider;
    /**
     * The access key for API operations. You can retrieve this
     * from the 'Security & Credentials' section of the AWS console.
     */
    readonly accessKey: pulumi.Output<string | undefined>;
    /**
     * File containing custom root and intermediate certificates. Can also be configured using the `AWS_CA_BUNDLE` environment variable. (Setting `caBundle` in the shared config file is not supported.)
     */
    readonly customCaBundle: pulumi.Output<string | undefined>;
    /**
     * Address of the EC2 metadata service endpoint to use. Can also be configured using the `AWS_EC2_METADATA_SERVICE_ENDPOINT` environment variable.
     */
    readonly ec2MetadataServiceEndpoint: pulumi.Output<string | undefined>;
    /**
     * Protocol to use with EC2 metadata service endpoint.Valid values are `IPv4` and `IPv6`. Can also be configured using the `AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE` environment variable.
     */
    readonly ec2MetadataServiceEndpointMode: pulumi.Output<string | undefined>;
    /**
     * URL of a proxy to use for HTTP requests when accessing the AWS API. Can also be set using the `HTTP_PROXY` or `httpProxy` environment variables.
     */
    readonly httpProxy: pulumi.Output<string | undefined>;
    /**
     * URL of a proxy to use for HTTPS requests when accessing the AWS API. Can also be set using the `HTTPS_PROXY` or `httpsProxy` environment variables.
     */
    readonly httpsProxy: pulumi.Output<string | undefined>;
    /**
     * Comma-separated list of hosts that should not use HTTP or HTTPS proxies. Can also be set using the `NO_PROXY` or `noProxy` environment variables.
     */
    readonly noProxy: pulumi.Output<string | undefined>;
    /**
     * The profile for API operations. If not set, the default profile
     * created with `aws configure` will be used.
     */
    readonly profile: pulumi.Output<string | undefined>;
    /**
     * The region where AWS operations will take place. Examples
     * are us-east-1, us-west-2, etc.
     */
    readonly region: pulumi.Output<string | undefined>;
    /**
     * Specifies how retries are attempted. Valid values are `standard` and `adaptive`. Can also be configured using the `AWS_RETRY_MODE` environment variable.
     */
    readonly retryMode: pulumi.Output<string | undefined>;
    /**
     * Specifies whether S3 API calls in the `us-east-1` region use the legacy global endpoint or a regional endpoint. Valid values are `legacy` or `regional`. Can also be configured using the `AWS_S3_US_EAST_1_REGIONAL_ENDPOINT` environment variable or the `s3UsEast1RegionalEndpoint` shared config file parameter
     */
    readonly s3UsEast1RegionalEndpoint: pulumi.Output<string | undefined>;
    /**
     * The secret key for API operations. You can retrieve this
     * from the 'Security & Credentials' section of the AWS console.
     */
    readonly secretKey: pulumi.Output<string | undefined>;
    /**
     * The region where AWS STS operations will take place. Examples
     * are us-east-1 and us-west-2.
     */
    readonly stsRegion: pulumi.Output<string | undefined>;
    /**
     * session token. A session token is only required if you are
     * using temporary security credentials.
     */
    readonly token: pulumi.Output<string | undefined>;
    /**
     * Create a Provider resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ProviderArgs, opts?: pulumi.ResourceOptions);
    /**
     * This function returns a Terraform config object with terraform-namecased keys,to be used with the Terraform Module Provider.
     */
    terraformConfig(): pulumi.Output<Provider.TerraformConfigResult>;
}
/**
 * The set of arguments for constructing a Provider resource.
 */
export interface ProviderArgs {
    /**
     * The access key for API operations. You can retrieve this
     * from the 'Security & Credentials' section of the AWS console.
     */
    accessKey?: pulumi.Input<string>;
    allowedAccountIds?: pulumi.Input<pulumi.Input<string>[]>;
    assumeRoleWithWebIdentity?: pulumi.Input<inputs.ProviderAssumeRoleWithWebIdentity>;
    assumeRoles?: pulumi.Input<pulumi.Input<inputs.ProviderAssumeRole>[]>;
    /**
     * File containing custom root and intermediate certificates. Can also be configured using the `AWS_CA_BUNDLE` environment variable. (Setting `caBundle` in the shared config file is not supported.)
     */
    customCaBundle?: pulumi.Input<string>;
    /**
     * Configuration block with settings to default resource tags across all resources.
     */
    defaultTags?: pulumi.Input<inputs.ProviderDefaultTags>;
    /**
     * Address of the EC2 metadata service endpoint to use. Can also be configured using the `AWS_EC2_METADATA_SERVICE_ENDPOINT` environment variable.
     */
    ec2MetadataServiceEndpoint?: pulumi.Input<string>;
    /**
     * Protocol to use with EC2 metadata service endpoint.Valid values are `IPv4` and `IPv6`. Can also be configured using the `AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE` environment variable.
     */
    ec2MetadataServiceEndpointMode?: pulumi.Input<string>;
    endpoints?: pulumi.Input<pulumi.Input<inputs.ProviderEndpoint>[]>;
    forbiddenAccountIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * URL of a proxy to use for HTTP requests when accessing the AWS API. Can also be set using the `HTTP_PROXY` or `httpProxy` environment variables.
     */
    httpProxy?: pulumi.Input<string>;
    /**
     * URL of a proxy to use for HTTPS requests when accessing the AWS API. Can also be set using the `HTTPS_PROXY` or `httpsProxy` environment variables.
     */
    httpsProxy?: pulumi.Input<string>;
    /**
     * Configuration block with settings to ignore resource tags across all resources.
     */
    ignoreTags?: pulumi.Input<inputs.ProviderIgnoreTags>;
    /**
     * Explicitly allow the provider to perform "insecure" SSL requests. If omitted, default value is `false`
     */
    insecure?: pulumi.Input<boolean>;
    /**
     * The maximum number of times an AWS API request is
     * being executed. If the API request still fails, an error is
     * thrown.
     */
    maxRetries?: pulumi.Input<number>;
    /**
     * Comma-separated list of hosts that should not use HTTP or HTTPS proxies. Can also be set using the `NO_PROXY` or `noProxy` environment variables.
     */
    noProxy?: pulumi.Input<string>;
    /**
     * The profile for API operations. If not set, the default profile
     * created with `aws configure` will be used.
     */
    profile?: pulumi.Input<string>;
    /**
     * The region where AWS operations will take place. Examples
     * are us-east-1, us-west-2, etc.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies how retries are attempted. Valid values are `standard` and `adaptive`. Can also be configured using the `AWS_RETRY_MODE` environment variable.
     */
    retryMode?: pulumi.Input<string>;
    /**
     * Specifies whether S3 API calls in the `us-east-1` region use the legacy global endpoint or a regional endpoint. Valid values are `legacy` or `regional`. Can also be configured using the `AWS_S3_US_EAST_1_REGIONAL_ENDPOINT` environment variable or the `s3UsEast1RegionalEndpoint` shared config file parameter
     */
    s3UsEast1RegionalEndpoint?: pulumi.Input<string>;
    /**
     * Set this to true to enable the request to use path-style addressing,
     * i.e., https://s3.amazonaws.com/BUCKET/KEY. By default, the S3 client will
     * use virtual hosted bucket addressing when possible
     * (https://BUCKET.s3.amazonaws.com/KEY). Specific to the Amazon S3 service.
     */
    s3UsePathStyle?: pulumi.Input<boolean>;
    /**
     * The secret key for API operations. You can retrieve this
     * from the 'Security & Credentials' section of the AWS console.
     */
    secretKey?: pulumi.Input<string>;
    /**
     * List of paths to shared config files. If not set, defaults to [~/.aws/config].
     */
    sharedConfigFiles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of paths to shared credentials files. If not set, defaults to [~/.aws/credentials].
     */
    sharedCredentialsFiles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Skip the credentials validation via STS API. Used for AWS API implementations that do not have STS available/implemented.
     */
    skipCredentialsValidation?: pulumi.Input<boolean>;
    /**
     * Skip the AWS Metadata API check. Used for AWS API implementations that do not have a metadata api endpoint.
     */
    skipMetadataApiCheck?: pulumi.Input<boolean>;
    /**
     * Skip static validation of region name. Used by users of alternative AWS-like APIs or users w/ access to regions that are not public (yet).
     */
    skipRegionValidation?: pulumi.Input<boolean>;
    /**
     * Skip requesting the account ID. Used for AWS API implementations that do not have IAM/STS API and/or metadata API.
     */
    skipRequestingAccountId?: pulumi.Input<boolean>;
    /**
     * The region where AWS STS operations will take place. Examples
     * are us-east-1 and us-west-2.
     */
    stsRegion?: pulumi.Input<string>;
    /**
     * session token. A session token is only required if you are
     * using temporary security credentials.
     */
    token?: pulumi.Input<string>;
    /**
     * The capacity of the AWS SDK's token bucket rate limiter.
     */
    tokenBucketRateLimiterCapacity?: pulumi.Input<number>;
    /**
     * Resolve an endpoint with DualStack capability
     */
    useDualstackEndpoint?: pulumi.Input<boolean>;
    /**
     * Resolve an endpoint with FIPS capability
     */
    useFipsEndpoint?: pulumi.Input<boolean>;
}
export declare namespace Provider {
    /**
     * The results of the Provider.terraformConfig method.
     */
    interface TerraformConfigResult {
        readonly result: {
            [key: string]: any;
        };
    }
}

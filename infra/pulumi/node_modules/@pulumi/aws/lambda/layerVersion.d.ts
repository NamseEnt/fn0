import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Lambda Layer Version. Use this resource to share code and dependencies across multiple Lambda functions.
 *
 * For information about Lambda Layers and how to use them, see [AWS Lambda Layers](https://docs.aws.amazon.com/lambda/latest/dg/configuration-layers.html).
 *
 * > **Note:** Setting `skipDestroy` to `true` means that the AWS Provider will not destroy any layer version, even when running `pulumi destroy`. Layer versions are thus intentional dangling resources that are not managed by Pulumi and may incur extra expense in your AWS account.
 *
 * ## Example Usage
 *
 * ### Basic Layer
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.LayerVersion("example", {
 *     code: new pulumi.asset.FileArchive("lambda_layer_payload.zip"),
 *     layerName: "lambda_layer_name",
 *     compatibleRuntimes: ["nodejs20.x"],
 * });
 * ```
 *
 * ### Layer with S3 Source
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.LayerVersion("example", {
 *     s3Bucket: lambdaLayerZip.bucket,
 *     s3Key: lambdaLayerZip.key,
 *     layerName: "lambda_layer_name",
 *     compatibleRuntimes: [
 *         "nodejs20.x",
 *         "python3.12",
 *     ],
 *     compatibleArchitectures: [
 *         "x86_64",
 *         "arm64",
 *     ],
 * });
 * ```
 *
 * ### Layer with Multiple Runtimes and Architectures
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.lambda.LayerVersion("example", {
 *     code: new pulumi.asset.FileArchive("lambda_layer_payload.zip"),
 *     layerName: "multi_runtime_layer",
 *     description: "Shared utilities for Lambda functions",
 *     licenseInfo: "MIT",
 *     sourceCodeHash: std.filebase64sha256({
 *         input: "lambda_layer_payload.zip",
 *     }).then(invoke => invoke.result),
 *     compatibleRuntimes: [
 *         "nodejs18.x",
 *         "nodejs20.x",
 *         "python3.11",
 *         "python3.12",
 *     ],
 *     compatibleArchitectures: [
 *         "x86_64",
 *         "arm64",
 *     ],
 * });
 * ```
 *
 * ## Specifying the Deployment Package
 *
 * AWS Lambda Layers expect source code to be provided as a deployment package whose structure varies depending on which `compatibleRuntimes` this layer specifies. See [Runtimes](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleRuntimes) for the valid values of `compatibleRuntimes`.
 *
 * Once you have created your deployment package you can specify it either directly as a local file (using the `filename` argument) or indirectly via Amazon S3 (using the `s3Bucket`, `s3Key` and `s3ObjectVersion` arguments). When providing the deployment package via S3 it may be useful to use the `aws.s3.BucketObjectv2` resource to upload it.
 *
 * For larger deployment packages it is recommended by Amazon to upload via S3, since the S3 API has better support for uploading large files efficiently.
 *
 * ## Import
 *
 * Using `pulumi import`, import Lambda Layers using `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:lambda/layerVersion:LayerVersion example arn:aws:lambda:us-west-2:123456789012:layer:example:1
 * ```
 */
export declare class LayerVersion extends pulumi.CustomResource {
    /**
     * Get an existing LayerVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LayerVersionState, opts?: pulumi.CustomResourceOptions): LayerVersion;
    /**
     * Returns true if the given object is an instance of LayerVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LayerVersion;
    /**
     * ARN of the Lambda Layer with version.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Path to the function's deployment package within the local filesystem. If defined, The `s3_`-prefixed options cannot be used.
     */
    readonly code: pulumi.Output<pulumi.asset.Archive | undefined>;
    /**
     * Base64-encoded representation of raw SHA-256 sum of the zip file.
     */
    readonly codeSha256: pulumi.Output<string>;
    /**
     * List of [Architectures](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleArchitectures) this layer is compatible with. Currently `x8664` and `arm64` can be specified.
     */
    readonly compatibleArchitectures: pulumi.Output<string[] | undefined>;
    /**
     * List of [Runtimes](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleRuntimes) this layer is compatible with. Up to 15 runtimes can be specified.
     */
    readonly compatibleRuntimes: pulumi.Output<string[] | undefined>;
    /**
     * Date this resource was created.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Description of what your Lambda Layer does.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the Lambda Layer without version.
     */
    readonly layerArn: pulumi.Output<string>;
    /**
     * Unique name for your Lambda Layer.
     *
     * The following arguments are optional:
     */
    readonly layerName: pulumi.Output<string>;
    /**
     * License info for your Lambda Layer. See [License Info](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-LicenseInfo).
     */
    readonly licenseInfo: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * S3 bucket location containing the function's deployment package. Conflicts with `filename`. This bucket must reside in the same AWS region where you are creating the Lambda function.
     */
    readonly s3Bucket: pulumi.Output<string | undefined>;
    /**
     * S3 key of an object containing the function's deployment package. Conflicts with `filename`.
     */
    readonly s3Key: pulumi.Output<string | undefined>;
    /**
     * Object version containing the function's deployment package. Conflicts with `filename`.
     */
    readonly s3ObjectVersion: pulumi.Output<string | undefined>;
    /**
     * ARN of a signing job.
     */
    readonly signingJobArn: pulumi.Output<string>;
    /**
     * ARN for a signing profile version.
     */
    readonly signingProfileVersionArn: pulumi.Output<string>;
    /**
     * Whether to retain the old version of a previously deployed Lambda Layer. Default is `false`. When this is not set to `true`, changing any of `compatibleArchitectures`, `compatibleRuntimes`, `description`, `filename`, `layerName`, `licenseInfo`, `s3Bucket`, `s3Key`, `s3ObjectVersion`, or `sourceCodeHash` forces deletion of the existing layer version and creation of a new layer version.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Virtual attribute used to trigger replacement when source code changes. Must be set to a base64-encoded SHA256 hash of the package file specified with either `filename` or `s3Key`. The usual way to set this is `filebase64sha256("file.zip")` or `base64sha256(file("file.zip"))`, where "file.zip" is the local filename of the lambda layer source archive.
     */
    readonly sourceCodeHash: pulumi.Output<string>;
    /**
     * Size in bytes of the function .zip file.
     */
    readonly sourceCodeSize: pulumi.Output<number>;
    /**
     * Lambda Layer version.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a LayerVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LayerVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LayerVersion resources.
 */
export interface LayerVersionState {
    /**
     * ARN of the Lambda Layer with version.
     */
    arn?: pulumi.Input<string>;
    /**
     * Path to the function's deployment package within the local filesystem. If defined, The `s3_`-prefixed options cannot be used.
     */
    code?: pulumi.Input<pulumi.asset.Archive>;
    /**
     * Base64-encoded representation of raw SHA-256 sum of the zip file.
     */
    codeSha256?: pulumi.Input<string>;
    /**
     * List of [Architectures](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleArchitectures) this layer is compatible with. Currently `x8664` and `arm64` can be specified.
     */
    compatibleArchitectures?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of [Runtimes](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleRuntimes) this layer is compatible with. Up to 15 runtimes can be specified.
     */
    compatibleRuntimes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Date this resource was created.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Description of what your Lambda Layer does.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the Lambda Layer without version.
     */
    layerArn?: pulumi.Input<string>;
    /**
     * Unique name for your Lambda Layer.
     *
     * The following arguments are optional:
     */
    layerName?: pulumi.Input<string>;
    /**
     * License info for your Lambda Layer. See [License Info](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-LicenseInfo).
     */
    licenseInfo?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * S3 bucket location containing the function's deployment package. Conflicts with `filename`. This bucket must reside in the same AWS region where you are creating the Lambda function.
     */
    s3Bucket?: pulumi.Input<string>;
    /**
     * S3 key of an object containing the function's deployment package. Conflicts with `filename`.
     */
    s3Key?: pulumi.Input<string>;
    /**
     * Object version containing the function's deployment package. Conflicts with `filename`.
     */
    s3ObjectVersion?: pulumi.Input<string>;
    /**
     * ARN of a signing job.
     */
    signingJobArn?: pulumi.Input<string>;
    /**
     * ARN for a signing profile version.
     */
    signingProfileVersionArn?: pulumi.Input<string>;
    /**
     * Whether to retain the old version of a previously deployed Lambda Layer. Default is `false`. When this is not set to `true`, changing any of `compatibleArchitectures`, `compatibleRuntimes`, `description`, `filename`, `layerName`, `licenseInfo`, `s3Bucket`, `s3Key`, `s3ObjectVersion`, or `sourceCodeHash` forces deletion of the existing layer version and creation of a new layer version.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Virtual attribute used to trigger replacement when source code changes. Must be set to a base64-encoded SHA256 hash of the package file specified with either `filename` or `s3Key`. The usual way to set this is `filebase64sha256("file.zip")` or `base64sha256(file("file.zip"))`, where "file.zip" is the local filename of the lambda layer source archive.
     */
    sourceCodeHash?: pulumi.Input<string>;
    /**
     * Size in bytes of the function .zip file.
     */
    sourceCodeSize?: pulumi.Input<number>;
    /**
     * Lambda Layer version.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LayerVersion resource.
 */
export interface LayerVersionArgs {
    /**
     * Path to the function's deployment package within the local filesystem. If defined, The `s3_`-prefixed options cannot be used.
     */
    code?: pulumi.Input<pulumi.asset.Archive>;
    /**
     * List of [Architectures](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleArchitectures) this layer is compatible with. Currently `x8664` and `arm64` can be specified.
     */
    compatibleArchitectures?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of [Runtimes](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-CompatibleRuntimes) this layer is compatible with. Up to 15 runtimes can be specified.
     */
    compatibleRuntimes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Description of what your Lambda Layer does.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique name for your Lambda Layer.
     *
     * The following arguments are optional:
     */
    layerName: pulumi.Input<string>;
    /**
     * License info for your Lambda Layer. See [License Info](https://docs.aws.amazon.com/lambda/latest/dg/API_PublishLayerVersion.html#SSS-PublishLayerVersion-request-LicenseInfo).
     */
    licenseInfo?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * S3 bucket location containing the function's deployment package. Conflicts with `filename`. This bucket must reside in the same AWS region where you are creating the Lambda function.
     */
    s3Bucket?: pulumi.Input<string>;
    /**
     * S3 key of an object containing the function's deployment package. Conflicts with `filename`.
     */
    s3Key?: pulumi.Input<string>;
    /**
     * Object version containing the function's deployment package. Conflicts with `filename`.
     */
    s3ObjectVersion?: pulumi.Input<string>;
    /**
     * Whether to retain the old version of a previously deployed Lambda Layer. Default is `false`. When this is not set to `true`, changing any of `compatibleArchitectures`, `compatibleRuntimes`, `description`, `filename`, `layerName`, `licenseInfo`, `s3Bucket`, `s3Key`, `s3ObjectVersion`, or `sourceCodeHash` forces deletion of the existing layer version and creation of a new layer version.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Virtual attribute used to trigger replacement when source code changes. Must be set to a base64-encoded SHA256 hash of the package file specified with either `filename` or `s3Key`. The usual way to set this is `filebase64sha256("file.zip")` or `base64sha256(file("file.zip"))`, where "file.zip" is the local filename of the lambda layer source archive.
     */
    sourceCodeHash?: pulumi.Input<string>;
}

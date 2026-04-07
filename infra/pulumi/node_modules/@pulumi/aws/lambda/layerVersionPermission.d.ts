import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Lambda Layer Version Permission. Use this resource to share Lambda Layers with other AWS accounts, organizations, or make them publicly accessible.
 *
 * For information about Lambda Layer Permissions and how to use them, see [Using Resource-based Policies for AWS Lambda](https://docs.aws.amazon.com/lambda/latest/dg/access-control-resource-based.html#permissions-resource-xaccountlayer).
 *
 * > **Note:** Setting `skipDestroy` to `true` means that the AWS Provider will not destroy any layer version permission, even when running `pulumi destroy`. Layer version permissions are thus intentional dangling resources that are not managed by Pulumi and may incur extra expense in your AWS account.
 *
 * ## Example Usage
 *
 * ### Share Layer with Specific Account
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Lambda layer to share
 * const example = new aws.lambda.LayerVersion("example", {
 *     code: new pulumi.asset.FileArchive("layer.zip"),
 *     layerName: "shared_utilities",
 *     description: "Common utilities for Lambda functions",
 *     compatibleRuntimes: [
 *         "nodejs20.x",
 *         "python3.12",
 *     ],
 * });
 * // Grant permission to specific AWS account
 * const exampleLayerVersionPermission = new aws.lambda.LayerVersionPermission("example", {
 *     layerName: example.layerName,
 *     versionNumber: example.version,
 *     principal: "123456789012",
 *     action: "lambda:GetLayerVersion",
 *     statementId: "dev-account-access",
 * });
 * ```
 *
 * ### Share Layer with Organization
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.LayerVersionPermission("example", {
 *     layerName: exampleAwsLambdaLayerVersion.layerName,
 *     versionNumber: exampleAwsLambdaLayerVersion.version,
 *     principal: "*",
 *     organizationId: "o-1234567890",
 *     action: "lambda:GetLayerVersion",
 *     statementId: "org-wide-access",
 * });
 * ```
 *
 * ### Share Layer Publicly
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.LayerVersionPermission("example", {
 *     layerName: exampleAwsLambdaLayerVersion.layerName,
 *     versionNumber: exampleAwsLambdaLayerVersion.version,
 *     principal: "*",
 *     action: "lambda:GetLayerVersion",
 *     statementId: "public-access",
 * });
 * ```
 *
 * ### Multiple Account Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Share with multiple specific accounts
 * const devAccount = new aws.lambda.LayerVersionPermission("dev_account", {
 *     layerName: example.layerName,
 *     versionNumber: example.version,
 *     principal: "111111111111",
 *     action: "lambda:GetLayerVersion",
 *     statementId: "dev-account",
 * });
 * const stagingAccount = new aws.lambda.LayerVersionPermission("staging_account", {
 *     layerName: example.layerName,
 *     versionNumber: example.version,
 *     principal: "222222222222",
 *     action: "lambda:GetLayerVersion",
 *     statementId: "staging-account",
 * });
 * const prodAccount = new aws.lambda.LayerVersionPermission("prod_account", {
 *     layerName: example.layerName,
 *     versionNumber: example.version,
 *     principal: "333333333333",
 *     action: "lambda:GetLayerVersion",
 *     statementId: "prod-account",
 * });
 * ```
 *
 * ## Import
 *
 * For backwards compatibility, the following legacy `pulumi import` command is also supported:
 *
 * ```sh
 * $ pulumi import aws:lambda/layerVersionPermission:LayerVersionPermission example arn:aws:lambda:us-west-2:123456789012:layer:shared_utilities,1
 * ```
 */
export declare class LayerVersionPermission extends pulumi.CustomResource {
    /**
     * Get an existing LayerVersionPermission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LayerVersionPermissionState, opts?: pulumi.CustomResourceOptions): LayerVersionPermission;
    /**
     * Returns true if the given object is an instance of LayerVersionPermission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LayerVersionPermission;
    /**
     * Action that will be allowed. `lambda:GetLayerVersion` is the standard value for layer access.
     */
    readonly action: pulumi.Output<string>;
    /**
     * Name or ARN of the Lambda Layer.
     */
    readonly layerName: pulumi.Output<string>;
    /**
     * AWS Organization ID that should be able to use your Lambda Layer. `principal` should be set to `*` when `organizationId` is provided.
     */
    readonly organizationId: pulumi.Output<string | undefined>;
    /**
     * Full Lambda Layer Permission policy.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * AWS account ID that should be able to use your Lambda Layer. Use `*` to share with all AWS accounts.
     */
    readonly principal: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Unique identifier for the current revision of the policy.
     */
    readonly revisionId: pulumi.Output<string>;
    /**
     * Whether to retain the permission when the resource is destroyed. Default is `false`.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Unique identifier for the permission statement.
     */
    readonly statementId: pulumi.Output<string>;
    /**
     * Version of Lambda Layer to grant access to. Note: permissions only apply to a single version of a layer.
     *
     * The following arguments are optional:
     */
    readonly versionNumber: pulumi.Output<number>;
    /**
     * Create a LayerVersionPermission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LayerVersionPermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LayerVersionPermission resources.
 */
export interface LayerVersionPermissionState {
    /**
     * Action that will be allowed. `lambda:GetLayerVersion` is the standard value for layer access.
     */
    action?: pulumi.Input<string>;
    /**
     * Name or ARN of the Lambda Layer.
     */
    layerName?: pulumi.Input<string>;
    /**
     * AWS Organization ID that should be able to use your Lambda Layer. `principal` should be set to `*` when `organizationId` is provided.
     */
    organizationId?: pulumi.Input<string>;
    /**
     * Full Lambda Layer Permission policy.
     */
    policy?: pulumi.Input<string>;
    /**
     * AWS account ID that should be able to use your Lambda Layer. Use `*` to share with all AWS accounts.
     */
    principal?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Unique identifier for the current revision of the policy.
     */
    revisionId?: pulumi.Input<string>;
    /**
     * Whether to retain the permission when the resource is destroyed. Default is `false`.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Unique identifier for the permission statement.
     */
    statementId?: pulumi.Input<string>;
    /**
     * Version of Lambda Layer to grant access to. Note: permissions only apply to a single version of a layer.
     *
     * The following arguments are optional:
     */
    versionNumber?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a LayerVersionPermission resource.
 */
export interface LayerVersionPermissionArgs {
    /**
     * Action that will be allowed. `lambda:GetLayerVersion` is the standard value for layer access.
     */
    action: pulumi.Input<string>;
    /**
     * Name or ARN of the Lambda Layer.
     */
    layerName: pulumi.Input<string>;
    /**
     * AWS Organization ID that should be able to use your Lambda Layer. `principal` should be set to `*` when `organizationId` is provided.
     */
    organizationId?: pulumi.Input<string>;
    /**
     * AWS account ID that should be able to use your Lambda Layer. Use `*` to share with all AWS accounts.
     */
    principal: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the permission when the resource is destroyed. Default is `false`.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Unique identifier for the permission statement.
     */
    statementId: pulumi.Input<string>;
    /**
     * Version of Lambda Layer to grant access to. Note: permissions only apply to a single version of a layer.
     *
     * The following arguments are optional:
     */
    versionNumber: pulumi.Input<number>;
}

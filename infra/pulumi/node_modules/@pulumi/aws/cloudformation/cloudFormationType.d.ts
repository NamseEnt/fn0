import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a version of a CloudFormation Type.
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_cloudformation_type` using the type version Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/cloudFormationType:CloudFormationType example arn:aws:cloudformation:us-east-1:123456789012:type/resource/ExampleCompany-ExampleService-ExampleType/1
 * ```
 */
export declare class CloudFormationType extends pulumi.CustomResource {
    /**
     * Get an existing CloudFormationType resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CloudFormationTypeState, opts?: pulumi.CustomResourceOptions): CloudFormationType;
    /**
     * Returns true if the given object is an instance of CloudFormationType.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CloudFormationType;
    /**
     * (Optional) Amazon Resource Name (ARN) of the CloudFormation Type version. See also `typeArn`.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Identifier of the CloudFormation Type default version.
     */
    readonly defaultVersionId: pulumi.Output<string>;
    /**
     * Deprecation status of the version.
     */
    readonly deprecatedStatus: pulumi.Output<string>;
    /**
     * Description of the version.
     */
    readonly description: pulumi.Output<string>;
    /**
     * URL of the documentation for the CloudFormation Type.
     */
    readonly documentationUrl: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role for CloudFormation to assume when invoking the extension. If your extension calls AWS APIs in any of its handlers, you must create an IAM execution role that includes the necessary permissions to call those AWS APIs, and provision that execution role in your account. When CloudFormation needs to invoke the extension handler, CloudFormation assumes this execution role to create a temporary session token, which it then passes to the extension handler, thereby supplying your extension with the appropriate credentials.
     */
    readonly executionRoleArn: pulumi.Output<string | undefined>;
    /**
     * Whether the CloudFormation Type version is the default version.
     */
    readonly isDefaultVersion: pulumi.Output<boolean>;
    /**
     * Configuration block containing logging configuration.
     */
    readonly loggingConfig: pulumi.Output<outputs.cloudformation.CloudFormationTypeLoggingConfig | undefined>;
    /**
     * Provisioning behavior of the CloudFormation Type.
     */
    readonly provisioningType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * JSON document of the CloudFormation Type schema.
     */
    readonly schema: pulumi.Output<string>;
    /**
     * URL to the S3 bucket containing the extension project package that contains the necessary files for the extension you want to register. Must begin with `s3://` or `https://`. For example, `s3://example-bucket/example-object`.
     */
    readonly schemaHandlerPackage: pulumi.Output<string>;
    /**
     * URL of the source code for the CloudFormation Type.
     */
    readonly sourceUrl: pulumi.Output<string>;
    /**
     * CloudFormation Registry Type. For example, `RESOURCE` or `MODULE`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * (Optional) Amazon Resource Name (ARN) of the CloudFormation Type. See also `arn`.
     */
    readonly typeArn: pulumi.Output<string>;
    /**
     * CloudFormation Type name. For example, `ExampleCompany::ExampleService::ExampleResource`.
     */
    readonly typeName: pulumi.Output<string>;
    /**
     * (Optional) Identifier of the CloudFormation Type version.
     */
    readonly versionId: pulumi.Output<string>;
    /**
     * Scope of the CloudFormation Type.
     */
    readonly visibility: pulumi.Output<string>;
    /**
     * Create a CloudFormationType resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CloudFormationTypeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CloudFormationType resources.
 */
export interface CloudFormationTypeState {
    /**
     * (Optional) Amazon Resource Name (ARN) of the CloudFormation Type version. See also `typeArn`.
     */
    arn?: pulumi.Input<string>;
    /**
     * Identifier of the CloudFormation Type default version.
     */
    defaultVersionId?: pulumi.Input<string>;
    /**
     * Deprecation status of the version.
     */
    deprecatedStatus?: pulumi.Input<string>;
    /**
     * Description of the version.
     */
    description?: pulumi.Input<string>;
    /**
     * URL of the documentation for the CloudFormation Type.
     */
    documentationUrl?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role for CloudFormation to assume when invoking the extension. If your extension calls AWS APIs in any of its handlers, you must create an IAM execution role that includes the necessary permissions to call those AWS APIs, and provision that execution role in your account. When CloudFormation needs to invoke the extension handler, CloudFormation assumes this execution role to create a temporary session token, which it then passes to the extension handler, thereby supplying your extension with the appropriate credentials.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * Whether the CloudFormation Type version is the default version.
     */
    isDefaultVersion?: pulumi.Input<boolean>;
    /**
     * Configuration block containing logging configuration.
     */
    loggingConfig?: pulumi.Input<inputs.cloudformation.CloudFormationTypeLoggingConfig>;
    /**
     * Provisioning behavior of the CloudFormation Type.
     */
    provisioningType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * JSON document of the CloudFormation Type schema.
     */
    schema?: pulumi.Input<string>;
    /**
     * URL to the S3 bucket containing the extension project package that contains the necessary files for the extension you want to register. Must begin with `s3://` or `https://`. For example, `s3://example-bucket/example-object`.
     */
    schemaHandlerPackage?: pulumi.Input<string>;
    /**
     * URL of the source code for the CloudFormation Type.
     */
    sourceUrl?: pulumi.Input<string>;
    /**
     * CloudFormation Registry Type. For example, `RESOURCE` or `MODULE`.
     */
    type?: pulumi.Input<string>;
    /**
     * (Optional) Amazon Resource Name (ARN) of the CloudFormation Type. See also `arn`.
     */
    typeArn?: pulumi.Input<string>;
    /**
     * CloudFormation Type name. For example, `ExampleCompany::ExampleService::ExampleResource`.
     */
    typeName?: pulumi.Input<string>;
    /**
     * (Optional) Identifier of the CloudFormation Type version.
     */
    versionId?: pulumi.Input<string>;
    /**
     * Scope of the CloudFormation Type.
     */
    visibility?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CloudFormationType resource.
 */
export interface CloudFormationTypeArgs {
    /**
     * Amazon Resource Name (ARN) of the IAM Role for CloudFormation to assume when invoking the extension. If your extension calls AWS APIs in any of its handlers, you must create an IAM execution role that includes the necessary permissions to call those AWS APIs, and provision that execution role in your account. When CloudFormation needs to invoke the extension handler, CloudFormation assumes this execution role to create a temporary session token, which it then passes to the extension handler, thereby supplying your extension with the appropriate credentials.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * Configuration block containing logging configuration.
     */
    loggingConfig?: pulumi.Input<inputs.cloudformation.CloudFormationTypeLoggingConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * URL to the S3 bucket containing the extension project package that contains the necessary files for the extension you want to register. Must begin with `s3://` or `https://`. For example, `s3://example-bucket/example-object`.
     */
    schemaHandlerPackage: pulumi.Input<string>;
    /**
     * CloudFormation Registry Type. For example, `RESOURCE` or `MODULE`.
     */
    type?: pulumi.Input<string>;
    /**
     * CloudFormation Type name. For example, `ExampleCompany::ExampleService::ExampleResource`.
     */
    typeName: pulumi.Input<string>;
}

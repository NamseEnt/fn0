import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about a CloudFormation Type.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudformation.getCloudFormationType({
 *     type: "RESOURCE",
 *     typeName: "AWS::Athena::WorkGroup",
 * });
 * ```
 */
export declare function getCloudFormationType(args?: GetCloudFormationTypeArgs, opts?: pulumi.InvokeOptions): Promise<GetCloudFormationTypeResult>;
/**
 * A collection of arguments for invoking getCloudFormationType.
 */
export interface GetCloudFormationTypeArgs {
    /**
     * ARN of the CloudFormation Type. For example, `arn:aws:cloudformation:us-west-2::type/resource/AWS-EC2-VPC`.
     */
    arn?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * CloudFormation Registry Type. For example, `RESOURCE`.
     */
    type?: string;
    /**
     * CloudFormation Type name. For example, `AWS::EC2::VPC`.
     */
    typeName?: string;
    /**
     * Identifier of the CloudFormation Type version.
     */
    versionId?: string;
}
/**
 * A collection of values returned by getCloudFormationType.
 */
export interface GetCloudFormationTypeResult {
    readonly arn: string;
    /**
     * Identifier of the CloudFormation Type default version.
     */
    readonly defaultVersionId: string;
    /**
     * Deprecation status of the CloudFormation Type.
     */
    readonly deprecatedStatus: string;
    /**
     * Description of the CloudFormation Type.
     */
    readonly description: string;
    /**
     * URL of the documentation for the CloudFormation Type.
     */
    readonly documentationUrl: string;
    /**
     * ARN of the IAM Role used to register the CloudFormation Type.
     */
    readonly executionRoleArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Whether the CloudFormation Type version is the default version.
     */
    readonly isDefaultVersion: boolean;
    /**
     * List of objects containing logging configuration.
     */
    readonly loggingConfigs: outputs.cloudformation.GetCloudFormationTypeLoggingConfig[];
    /**
     * Provisioning behavior of the CloudFormation Type.
     */
    readonly provisioningType: string;
    readonly region: string;
    /**
     * JSON document of the CloudFormation Type schema.
     */
    readonly schema: string;
    /**
     * URL of the source code for the CloudFormation Type.
     */
    readonly sourceUrl: string;
    readonly type: string;
    readonly typeArn: string;
    readonly typeName: string;
    readonly versionId?: string;
    /**
     * Scope of the CloudFormation Type.
     */
    readonly visibility: string;
}
/**
 * Provides details about a CloudFormation Type.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudformation.getCloudFormationType({
 *     type: "RESOURCE",
 *     typeName: "AWS::Athena::WorkGroup",
 * });
 * ```
 */
export declare function getCloudFormationTypeOutput(args?: GetCloudFormationTypeOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCloudFormationTypeResult>;
/**
 * A collection of arguments for invoking getCloudFormationType.
 */
export interface GetCloudFormationTypeOutputArgs {
    /**
     * ARN of the CloudFormation Type. For example, `arn:aws:cloudformation:us-west-2::type/resource/AWS-EC2-VPC`.
     */
    arn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * CloudFormation Registry Type. For example, `RESOURCE`.
     */
    type?: pulumi.Input<string>;
    /**
     * CloudFormation Type name. For example, `AWS::EC2::VPC`.
     */
    typeName?: pulumi.Input<string>;
    /**
     * Identifier of the CloudFormation Type version.
     */
    versionId?: pulumi.Input<string>;
}

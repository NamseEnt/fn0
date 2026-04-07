import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details for a Cloud Control API Resource. The reading of these resources is proxied through Cloud Control API handlers to the backend service.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudcontrol.getResource({
 *     identifier: "example",
 *     typeName: "AWS::ECS::Cluster",
 * });
 * ```
 */
export declare function getResource(args: GetResourceArgs, opts?: pulumi.InvokeOptions): Promise<GetResourceResult>;
/**
 * A collection of arguments for invoking getResource.
 */
export interface GetResourceArgs {
    /**
     * Identifier of the CloudFormation resource type. For example, `vpc-12345678`.
     */
    identifier: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ARN of the IAM Role to assume for operations.
     */
    roleArn?: string;
    /**
     * CloudFormation resource type name. For example, `AWS::EC2::VPC`.
     *
     * The following arguments are optional:
     */
    typeName: string;
    /**
     * Identifier of the CloudFormation resource type version.
     */
    typeVersionId?: string;
}
/**
 * A collection of values returned by getResource.
 */
export interface GetResourceResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly identifier: string;
    /**
     * JSON string matching the CloudFormation resource type schema with current configuration.
     */
    readonly properties: string;
    readonly region: string;
    readonly roleArn?: string;
    readonly typeName: string;
    readonly typeVersionId?: string;
}
/**
 * Provides details for a Cloud Control API Resource. The reading of these resources is proxied through Cloud Control API handlers to the backend service.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudcontrol.getResource({
 *     identifier: "example",
 *     typeName: "AWS::ECS::Cluster",
 * });
 * ```
 */
export declare function getResourceOutput(args: GetResourceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResourceResult>;
/**
 * A collection of arguments for invoking getResource.
 */
export interface GetResourceOutputArgs {
    /**
     * Identifier of the CloudFormation resource type. For example, `vpc-12345678`.
     */
    identifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM Role to assume for operations.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * CloudFormation resource type name. For example, `AWS::EC2::VPC`.
     *
     * The following arguments are optional:
     */
    typeName: pulumi.Input<string>;
    /**
     * Identifier of the CloudFormation resource type version.
     */
    typeVersionId?: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about a specific redshift subnet group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshift.getSubnetGroup({
 *     name: exampleAwsRedshiftSubnetGroup.name,
 * });
 * ```
 */
export declare function getSubnetGroup(args: GetSubnetGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetSubnetGroupResult>;
/**
 * A collection of arguments for invoking getSubnetGroup.
 */
export interface GetSubnetGroupArgs {
    /**
     * Name of the cluster subnet group for which information is requested.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Tags associated to the Subnet Group
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getSubnetGroup.
 */
export interface GetSubnetGroupResult {
    /**
     * ARN of the Redshift Subnet Group name.
     */
    readonly arn: string;
    /**
     * Description of the Redshift Subnet group.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    /**
     * An array of VPC subnet IDs.
     */
    readonly subnetIds: string[];
    /**
     * Tags associated to the Subnet Group
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides details about a specific redshift subnet group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshift.getSubnetGroup({
 *     name: exampleAwsRedshiftSubnetGroup.name,
 * });
 * ```
 */
export declare function getSubnetGroupOutput(args: GetSubnetGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSubnetGroupResult>;
/**
 * A collection of arguments for invoking getSubnetGroup.
 */
export interface GetSubnetGroupOutputArgs {
    /**
     * Name of the cluster subnet group for which information is requested.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags associated to the Subnet Group
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

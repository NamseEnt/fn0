import * as pulumi from "@pulumi/pulumi";
/**
 * Information about a database parameter group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.rds.getParameterGroup({
 *     name: "default.postgres15",
 * });
 * ```
 */
export declare function getParameterGroup(args: GetParameterGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetParameterGroupResult>;
/**
 * A collection of arguments for invoking getParameterGroup.
 */
export interface GetParameterGroupArgs {
    /**
     * DB parameter group name.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getParameterGroup.
 */
export interface GetParameterGroupResult {
    /**
     * ARN of the parameter group.
     */
    readonly arn: string;
    /**
     * Description of the parameter group.
     */
    readonly description: string;
    /**
     * Family of the parameter group.
     */
    readonly family: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
}
/**
 * Information about a database parameter group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.rds.getParameterGroup({
 *     name: "default.postgres15",
 * });
 * ```
 */
export declare function getParameterGroupOutput(args: GetParameterGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetParameterGroupResult>;
/**
 * A collection of arguments for invoking getParameterGroup.
 */
export interface GetParameterGroupOutputArgs {
    /**
     * DB parameter group name.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

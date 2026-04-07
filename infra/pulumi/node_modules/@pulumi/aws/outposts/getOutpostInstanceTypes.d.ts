import * as pulumi from "@pulumi/pulumi";
/**
 * Information about Outposts Instance Types.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getOutpostInstanceTypes({
 *     arn: exampleAwsOutpostsOutpost.arn,
 * });
 * ```
 */
export declare function getOutpostInstanceTypes(args: GetOutpostInstanceTypesArgs, opts?: pulumi.InvokeOptions): Promise<GetOutpostInstanceTypesResult>;
/**
 * A collection of arguments for invoking getOutpostInstanceTypes.
 */
export interface GetOutpostInstanceTypesArgs {
    /**
     * Outpost ARN.
     */
    arn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getOutpostInstanceTypes.
 */
export interface GetOutpostInstanceTypesResult {
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of instance types.
     */
    readonly instanceTypes: string[];
    readonly region: string;
}
/**
 * Information about Outposts Instance Types.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getOutpostInstanceTypes({
 *     arn: exampleAwsOutpostsOutpost.arn,
 * });
 * ```
 */
export declare function getOutpostInstanceTypesOutput(args: GetOutpostInstanceTypesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOutpostInstanceTypesResult>;
/**
 * A collection of arguments for invoking getOutpostInstanceTypes.
 */
export interface GetOutpostInstanceTypesOutputArgs {
    /**
     * Outpost ARN.
     */
    arn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

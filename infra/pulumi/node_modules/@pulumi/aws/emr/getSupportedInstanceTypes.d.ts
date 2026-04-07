import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS EMR Supported Instance Types.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.emr.getSupportedInstanceTypes({
 *     releaseLabel: "ebs-6.15.0",
 * });
 * ```
 *
 * ### With a Lifecycle Pre-Condition
 *
 * This data source can be used with a lifecycle precondition to ensure a given instance type is supported by EMR.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const instanceType = "r7g.large";
 * const releaseLabel = "emr-6.15.0";
 * const test = aws.emr.getSupportedInstanceTypes({
 *     releaseLabel: releaseLabel,
 * });
 * const testCluster = new aws.emr.Cluster("test", {
 *     releaseLabel: releaseLabel,
 *     masterInstanceGroup: {
 *         instanceType: instanceType,
 *     },
 * });
 * ```
 */
export declare function getSupportedInstanceTypes(args: GetSupportedInstanceTypesArgs, opts?: pulumi.InvokeOptions): Promise<GetSupportedInstanceTypesResult>;
/**
 * A collection of arguments for invoking getSupportedInstanceTypes.
 */
export interface GetSupportedInstanceTypesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Amazon EMR release label. For more information about Amazon EMR releases and their included application versions and features, see the [Amazon EMR Release Guide](https://docs.aws.amazon.com/emr/latest/ReleaseGuide/emr-release-components.html).
     */
    releaseLabel: string;
}
/**
 * A collection of values returned by getSupportedInstanceTypes.
 */
export interface GetSupportedInstanceTypesResult {
    readonly id: string;
    readonly region: string;
    readonly releaseLabel: string;
    /**
     * List of supported instance types. See `supportedInstanceTypes` below.
     */
    readonly supportedInstanceTypes: outputs.emr.GetSupportedInstanceTypesSupportedInstanceType[];
}
/**
 * Data source for managing AWS EMR Supported Instance Types.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.emr.getSupportedInstanceTypes({
 *     releaseLabel: "ebs-6.15.0",
 * });
 * ```
 *
 * ### With a Lifecycle Pre-Condition
 *
 * This data source can be used with a lifecycle precondition to ensure a given instance type is supported by EMR.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const instanceType = "r7g.large";
 * const releaseLabel = "emr-6.15.0";
 * const test = aws.emr.getSupportedInstanceTypes({
 *     releaseLabel: releaseLabel,
 * });
 * const testCluster = new aws.emr.Cluster("test", {
 *     releaseLabel: releaseLabel,
 *     masterInstanceGroup: {
 *         instanceType: instanceType,
 *     },
 * });
 * ```
 */
export declare function getSupportedInstanceTypesOutput(args: GetSupportedInstanceTypesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSupportedInstanceTypesResult>;
/**
 * A collection of arguments for invoking getSupportedInstanceTypes.
 */
export interface GetSupportedInstanceTypesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon EMR release label. For more information about Amazon EMR releases and their included application versions and features, see the [Amazon EMR Release Guide](https://docs.aws.amazon.com/emr/latest/ReleaseGuide/emr-release-components.html).
     */
    releaseLabel: pulumi.Input<string>;
}

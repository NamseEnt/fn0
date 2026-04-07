import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve EKS Clusters list
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.eks.getClusters({});
 * const exampleGetCluster = example.then(example => std.toset({
 *     input: example.names,
 * })).then(invoke => .reduce((__obj, [__key, __value]) => ({ ...__obj, [__key]: aws.eks.getCluster({
 *     name: __value,
 * }) })));
 * ```
 */
export declare function getClusters(args?: GetClustersArgs, opts?: pulumi.InvokeOptions): Promise<GetClustersResult>;
/**
 * A collection of arguments for invoking getClusters.
 */
export interface GetClustersArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getClusters.
 */
export interface GetClustersResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of EKS clusters names
     */
    readonly names: string[];
    readonly region: string;
}
/**
 * Retrieve EKS Clusters list
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.eks.getClusters({});
 * const exampleGetCluster = example.then(example => std.toset({
 *     input: example.names,
 * })).then(invoke => .reduce((__obj, [__key, __value]) => ({ ...__obj, [__key]: aws.eks.getCluster({
 *     name: __value,
 * }) })));
 * ```
 */
export declare function getClustersOutput(args?: GetClustersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClustersResult>;
/**
 * A collection of arguments for invoking getClusters.
 */
export interface GetClustersOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

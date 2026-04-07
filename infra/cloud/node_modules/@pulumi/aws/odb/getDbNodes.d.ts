import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for manging db nodes linked to cloud vm cluster of Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getDbNodes({
 *     cloudVmClusterId: "example",
 * });
 * ```
 */
export declare function getDbNodes(args: GetDbNodesArgs, opts?: pulumi.InvokeOptions): Promise<GetDbNodesResult>;
/**
 * A collection of arguments for invoking getDbNodes.
 */
export interface GetDbNodesArgs {
    /**
     * The unique identifier of the cloud vm cluster.
     *
     * The following arguments are optional:
     */
    cloudVmClusterId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDbNodes.
 */
export interface GetDbNodesResult {
    readonly cloudVmClusterId: string;
    /**
     * The list of DB nodes along with their properties.
     */
    readonly dbNodes: outputs.odb.GetDbNodesDbNode[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for manging db nodes linked to cloud vm cluster of Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getDbNodes({
 *     cloudVmClusterId: "example",
 * });
 * ```
 */
export declare function getDbNodesOutput(args: GetDbNodesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDbNodesResult>;
/**
 * A collection of arguments for invoking getDbNodes.
 */
export interface GetDbNodesOutputArgs {
    /**
     * The unique identifier of the cloud vm cluster.
     *
     * The following arguments are optional:
     */
    cloudVmClusterId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

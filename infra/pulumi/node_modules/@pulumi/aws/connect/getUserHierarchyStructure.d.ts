import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about a specific Amazon Connect User Hierarchy Structure
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.connect.getUserHierarchyStructure({
 *     instanceId: testAwsConnectInstance.id,
 * });
 * ```
 */
export declare function getUserHierarchyStructure(args: GetUserHierarchyStructureArgs, opts?: pulumi.InvokeOptions): Promise<GetUserHierarchyStructureResult>;
/**
 * A collection of arguments for invoking getUserHierarchyStructure.
 */
export interface GetUserHierarchyStructureArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getUserHierarchyStructure.
 */
export interface GetUserHierarchyStructureResult {
    /**
     * Block that defines the hierarchy structure's levels. The `hierarchyStructure` block is documented below.
     */
    readonly hierarchyStructures: outputs.connect.GetUserHierarchyStructureHierarchyStructure[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceId: string;
    readonly region: string;
}
/**
 * Provides details about a specific Amazon Connect User Hierarchy Structure
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.connect.getUserHierarchyStructure({
 *     instanceId: testAwsConnectInstance.id,
 * });
 * ```
 */
export declare function getUserHierarchyStructureOutput(args: GetUserHierarchyStructureOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUserHierarchyStructureResult>;
/**
 * A collection of arguments for invoking getUserHierarchyStructure.
 */
export interface GetUserHierarchyStructureOutputArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides information about a MemoryDB Parameter Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.memorydb.getParameterGroup({
 *     name: "my-parameter-group",
 * });
 * ```
 */
export declare function getParameterGroup(args: GetParameterGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetParameterGroupResult>;
/**
 * A collection of arguments for invoking getParameterGroup.
 */
export interface GetParameterGroupArgs {
    /**
     * Name of the parameter group.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the parameter group.
     */
    tags?: {
        [key: string]: string;
    };
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
     * Engine version that the parameter group can be used with.
     */
    readonly family: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Name of the parameter.
     */
    readonly name: string;
    /**
     * Set of user-defined MemoryDB parameters applied by the parameter group.
     */
    readonly parameters: outputs.memorydb.GetParameterGroupParameter[];
    readonly region: string;
    /**
     * Map of tags assigned to the parameter group.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides information about a MemoryDB Parameter Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.memorydb.getParameterGroup({
 *     name: "my-parameter-group",
 * });
 * ```
 */
export declare function getParameterGroupOutput(args: GetParameterGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetParameterGroupResult>;
/**
 * A collection of arguments for invoking getParameterGroup.
 */
export interface GetParameterGroupOutputArgs {
    /**
     * Name of the parameter group.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the parameter group.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

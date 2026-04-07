import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Use this data source to get the window IDs of SSM maintenance windows.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getMaintenanceWindows({
 *     filters: [{
 *         name: "Enabled",
 *         values: ["true"],
 *     }],
 * });
 * ```
 */
export declare function getMaintenanceWindows(args?: GetMaintenanceWindowsArgs, opts?: pulumi.InvokeOptions): Promise<GetMaintenanceWindowsResult>;
/**
 * A collection of arguments for invoking getMaintenanceWindows.
 */
export interface GetMaintenanceWindowsArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: inputs.ssm.GetMaintenanceWindowsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getMaintenanceWindows.
 */
export interface GetMaintenanceWindowsResult {
    readonly filters?: outputs.ssm.GetMaintenanceWindowsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of window IDs of the matched SSM maintenance windows.
     */
    readonly ids: string[];
    readonly region: string;
}
/**
 * Use this data source to get the window IDs of SSM maintenance windows.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getMaintenanceWindows({
 *     filters: [{
 *         name: "Enabled",
 *         values: ["true"],
 *     }],
 * });
 * ```
 */
export declare function getMaintenanceWindowsOutput(args?: GetMaintenanceWindowsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetMaintenanceWindowsResult>;
/**
 * A collection of arguments for invoking getMaintenanceWindows.
 */
export interface GetMaintenanceWindowsOutputArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ssm.GetMaintenanceWindowsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

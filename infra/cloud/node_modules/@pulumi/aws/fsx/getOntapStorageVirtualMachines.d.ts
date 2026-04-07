import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * This resource can be useful for getting back a set of FSx ONTAP Storage Virtual Machine (SVM) IDs.
 *
 * ## Example Usage
 *
 * The following shows outputting all SVM IDs for a given FSx ONTAP File System.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.fsx.getOntapStorageVirtualMachines({
 *     filters: [{
 *         name: "file-system-id",
 *         values: ["fs-12345678"],
 *     }],
 * });
 * ```
 */
export declare function getOntapStorageVirtualMachines(args?: GetOntapStorageVirtualMachinesArgs, opts?: pulumi.InvokeOptions): Promise<GetOntapStorageVirtualMachinesResult>;
/**
 * A collection of arguments for invoking getOntapStorageVirtualMachines.
 */
export interface GetOntapStorageVirtualMachinesArgs {
    /**
     * Configuration block. Detailed below.
     */
    filters?: inputs.fsx.GetOntapStorageVirtualMachinesFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getOntapStorageVirtualMachines.
 */
export interface GetOntapStorageVirtualMachinesResult {
    readonly filters?: outputs.fsx.GetOntapStorageVirtualMachinesFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of all SVM IDs found.
     */
    readonly ids: string[];
    readonly region: string;
}
/**
 * This resource can be useful for getting back a set of FSx ONTAP Storage Virtual Machine (SVM) IDs.
 *
 * ## Example Usage
 *
 * The following shows outputting all SVM IDs for a given FSx ONTAP File System.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.fsx.getOntapStorageVirtualMachines({
 *     filters: [{
 *         name: "file-system-id",
 *         values: ["fs-12345678"],
 *     }],
 * });
 * ```
 */
export declare function getOntapStorageVirtualMachinesOutput(args?: GetOntapStorageVirtualMachinesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOntapStorageVirtualMachinesResult>;
/**
 * A collection of arguments for invoking getOntapStorageVirtualMachines.
 */
export interface GetOntapStorageVirtualMachinesOutputArgs {
    /**
     * Configuration block. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.fsx.GetOntapStorageVirtualMachinesFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

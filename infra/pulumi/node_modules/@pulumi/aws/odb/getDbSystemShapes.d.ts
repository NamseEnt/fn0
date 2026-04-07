import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source to retrieve available system shapes Oracle Database@AWS.
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
 * const example = aws.odb.getDbSystemShapes({});
 * ```
 */
export declare function getDbSystemShapes(args?: GetDbSystemShapesArgs, opts?: pulumi.InvokeOptions): Promise<GetDbSystemShapesResult>;
/**
 * A collection of arguments for invoking getDbSystemShapes.
 */
export interface GetDbSystemShapesArgs {
    /**
     * The physical ID of the AZ, for example, use1-az4. This ID persists across accounts.
     */
    availabilityZoneId?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDbSystemShapes.
 */
export interface GetDbSystemShapesResult {
    readonly availabilityZoneId?: string;
    /**
     * The list of shapes and their properties. Information about a hardware system model (shape) that's available for an Exadata infrastructure. The shape determines resources, such as CPU cores, memory, and storage, to allocate to the Exadata infrastructure.
     */
    readonly dbSystemShapes: outputs.odb.GetDbSystemShapesDbSystemShape[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source to retrieve available system shapes Oracle Database@AWS.
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
 * const example = aws.odb.getDbSystemShapes({});
 * ```
 */
export declare function getDbSystemShapesOutput(args?: GetDbSystemShapesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDbSystemShapesResult>;
/**
 * A collection of arguments for invoking getDbSystemShapes.
 */
export interface GetDbSystemShapesOutputArgs {
    /**
     * The physical ID of the AZ, for example, use1-az4. This ID persists across accounts.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

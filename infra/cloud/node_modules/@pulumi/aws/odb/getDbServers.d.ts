import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for manging db servers linked to exadata infrastructure of Oracle Database@AWS.
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
 * const example = aws.odb.getDbServers({
 *     cloudExadataInfrastructureId: "exadata_infra_id",
 * });
 * ```
 */
export declare function getDbServers(args: GetDbServersArgs, opts?: pulumi.InvokeOptions): Promise<GetDbServersResult>;
/**
 * A collection of arguments for invoking getDbServers.
 */
export interface GetDbServersArgs {
    /**
     * The unique identifier of the cloud vm cluster.
     *
     * The following arguments are optional:
     */
    cloudExadataInfrastructureId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDbServers.
 */
export interface GetDbServersResult {
    readonly cloudExadataInfrastructureId: string;
    /**
     * the list of DB servers along with their properties.
     */
    readonly dbServers: outputs.odb.GetDbServersDbServer[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for manging db servers linked to exadata infrastructure of Oracle Database@AWS.
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
 * const example = aws.odb.getDbServers({
 *     cloudExadataInfrastructureId: "exadata_infra_id",
 * });
 * ```
 */
export declare function getDbServersOutput(args: GetDbServersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDbServersResult>;
/**
 * A collection of arguments for invoking getDbServers.
 */
export interface GetDbServersOutputArgs {
    /**
     * The unique identifier of the cloud vm cluster.
     *
     * The following arguments are optional:
     */
    cloudExadataInfrastructureId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

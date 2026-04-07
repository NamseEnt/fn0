import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source to retrieve available Grid Infrastructure versions of Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 */
export declare function getGiVersions(args?: GetGiVersionsArgs, opts?: pulumi.InvokeOptions): Promise<GetGiVersionsResult>;
/**
 * A collection of arguments for invoking getGiVersions.
 */
export interface GetGiVersionsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * The system shape.
     */
    shape?: string;
}
/**
 * A collection of values returned by getGiVersions.
 */
export interface GetGiVersionsResult {
    /**
     * Information about a specific version of Oracle Grid Infrastructure (GI) software that can be installed on a VM cluster.
     */
    readonly giVersions: outputs.odb.GetGiVersionsGiVersion[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly shape?: string;
}
/**
 * Data source to retrieve available Grid Infrastructure versions of Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 */
export declare function getGiVersionsOutput(args?: GetGiVersionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGiVersionsResult>;
/**
 * A collection of arguments for invoking getGiVersions.
 */
export interface GetGiVersionsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The system shape.
     */
    shape?: pulumi.Input<string>;
}

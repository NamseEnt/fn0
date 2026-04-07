import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for exadata infrastructures in AWS for Oracle Database@AWS.
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
 * const example = aws.odb.getCloudExadataInfrastructures({});
 * ```
 */
export declare function getCloudExadataInfrastructures(args?: GetCloudExadataInfrastructuresArgs, opts?: pulumi.InvokeOptions): Promise<GetCloudExadataInfrastructuresResult>;
/**
 * A collection of arguments for invoking getCloudExadataInfrastructures.
 */
export interface GetCloudExadataInfrastructuresArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getCloudExadataInfrastructures.
 */
export interface GetCloudExadataInfrastructuresResult {
    /**
     * List of Cloud Exadata Infrastructures. Returns basic information about the Cloud Exadata Infrastructures.
     */
    readonly cloudExadataInfrastructures: outputs.odb.GetCloudExadataInfrastructuresCloudExadataInfrastructure[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for exadata infrastructures in AWS for Oracle Database@AWS.
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
 * const example = aws.odb.getCloudExadataInfrastructures({});
 * ```
 */
export declare function getCloudExadataInfrastructuresOutput(args?: GetCloudExadataInfrastructuresOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCloudExadataInfrastructuresResult>;
/**
 * A collection of arguments for invoking getCloudExadataInfrastructures.
 */
export interface GetCloudExadataInfrastructuresOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

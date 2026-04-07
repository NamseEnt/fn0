import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about the AWS Direct Connect locations in the current AWS Region.
 * These are the locations that can be specified when configuring `aws.directconnect.Connection` or `aws.directconnect.LinkAggregationGroup` resources.
 *
 * > **Note:** This data source is different from the `aws.directconnect.getLocation` data source which retrieves information about a specific AWS Direct Connect location in the current AWS Region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.directconnect.getLocations({});
 * ```
 */
export declare function getLocations(args?: GetLocationsArgs, opts?: pulumi.InvokeOptions): Promise<GetLocationsResult>;
/**
 * A collection of arguments for invoking getLocations.
 */
export interface GetLocationsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getLocations.
 */
export interface GetLocationsResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Code for the locations.
     */
    readonly locationCodes: string[];
    readonly region: string;
}
/**
 * Retrieve information about the AWS Direct Connect locations in the current AWS Region.
 * These are the locations that can be specified when configuring `aws.directconnect.Connection` or `aws.directconnect.LinkAggregationGroup` resources.
 *
 * > **Note:** This data source is different from the `aws.directconnect.getLocation` data source which retrieves information about a specific AWS Direct Connect location in the current AWS Region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.directconnect.getLocations({});
 * ```
 */
export declare function getLocationsOutput(args?: GetLocationsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLocationsResult>;
/**
 * A collection of arguments for invoking getLocations.
 */
export interface GetLocationsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

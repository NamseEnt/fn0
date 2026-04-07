import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the ID of an [elastic beanstalk hosted zone](http://docs.aws.amazon.com/general/latest/gr/rande.html#elasticbeanstalk_region).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.elasticbeanstalk.getHostedZone({});
 * ```
 */
export declare function getHostedZone(args?: GetHostedZoneArgs, opts?: pulumi.InvokeOptions): Promise<GetHostedZoneResult>;
/**
 * A collection of arguments for invoking getHostedZone.
 */
export interface GetHostedZoneArgs {
    /**
     * Name of the Region whose hosted zone is desired. Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getHostedZone.
 */
export interface GetHostedZoneResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Use this data source to get the ID of an [elastic beanstalk hosted zone](http://docs.aws.amazon.com/general/latest/gr/rande.html#elasticbeanstalk_region).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.elasticbeanstalk.getHostedZone({});
 * ```
 */
export declare function getHostedZoneOutput(args?: GetHostedZoneOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetHostedZoneResult>;
/**
 * A collection of arguments for invoking getHostedZone.
 */
export interface GetHostedZoneOutputArgs {
    /**
     * Name of the Region whose hosted zone is desired. Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

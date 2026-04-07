import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about multiple Outposts.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getOutposts({
 *     siteId: id,
 * });
 * ```
 */
export declare function getOutposts(args?: GetOutpostsArgs, opts?: pulumi.InvokeOptions): Promise<GetOutpostsResult>;
/**
 * A collection of arguments for invoking getOutposts.
 */
export interface GetOutpostsArgs {
    /**
     * Availability Zone name.
     */
    availabilityZone?: string;
    /**
     * Availability Zone identifier.
     */
    availabilityZoneId?: string;
    /**
     * AWS Account identifier of the Outpost owner.
     */
    ownerId?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Site identifier.
     */
    siteId?: string;
}
/**
 * A collection of values returned by getOutposts.
 */
export interface GetOutpostsResult {
    /**
     * Set of Amazon Resource Names (ARNs).
     */
    readonly arns: string[];
    readonly availabilityZone: string;
    readonly availabilityZoneId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of identifiers.
     */
    readonly ids: string[];
    readonly ownerId: string;
    readonly region: string;
    readonly siteId: string;
}
/**
 * Provides details about multiple Outposts.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getOutposts({
 *     siteId: id,
 * });
 * ```
 */
export declare function getOutpostsOutput(args?: GetOutpostsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOutpostsResult>;
/**
 * A collection of arguments for invoking getOutposts.
 */
export interface GetOutpostsOutputArgs {
    /**
     * Availability Zone name.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Availability Zone identifier.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * AWS Account identifier of the Outpost owner.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Site identifier.
     */
    siteId?: pulumi.Input<string>;
}

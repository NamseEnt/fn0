import * as pulumi from "@pulumi/pulumi";
/**
 * This resource can be useful for getting back a list of Route53 Hosted Zone IDs for a Region.
 *
 * ## Example Usage
 *
 * The following example retrieves a list of all Hosted Zone IDs.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.route53.getZones({});
 * export const example = all.then(all => all.ids);
 * ```
 */
export declare function getZones(opts?: pulumi.InvokeOptions): Promise<GetZonesResult>;
/**
 * A collection of values returned by getZones.
 */
export interface GetZonesResult {
    readonly id: string;
    /**
     * A list of all the Route53 Hosted Zone IDs found.
     */
    readonly ids: string[];
}
/**
 * This resource can be useful for getting back a list of Route53 Hosted Zone IDs for a Region.
 *
 * ## Example Usage
 *
 * The following example retrieves a list of all Hosted Zone IDs.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.route53.getZones({});
 * export const example = all.then(all => all.ids);
 * ```
 */
export declare function getZonesOutput(opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetZonesResult>;

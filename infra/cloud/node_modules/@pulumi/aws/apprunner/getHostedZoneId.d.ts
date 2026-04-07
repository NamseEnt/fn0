import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the HostedZoneId of an AWS App Runner service deployed
 * in a given region for the purpose of using it in an AWS Route53 Alias record.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.apprunner.getHostedZoneId({});
 * const www = new aws.route53.Record("www", {
 *     zoneId: primary.zoneId,
 *     name: "example.com",
 *     type: aws.route53.RecordType.A,
 *     aliases: [{
 *         name: mainAwsApprunnerCustomDomainAssociation.dnsTarget,
 *         zoneId: main.then(main => main.id),
 *         evaluateTargetHealth: true,
 *     }],
 * });
 * ```
 */
export declare function getHostedZoneId(args?: GetHostedZoneIdArgs, opts?: pulumi.InvokeOptions): Promise<GetHostedZoneIdResult>;
/**
 * A collection of arguments for invoking getHostedZoneId.
 */
export interface GetHostedZoneIdArgs {
    /**
     * Name of the Region whose AWS App Runner service HostedZoneId is desired. Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getHostedZoneId.
 */
export interface GetHostedZoneIdResult {
    /**
     * ID of the AWS App Runner service HostedZoneId in the selected Region.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Use this data source to get the HostedZoneId of an AWS App Runner service deployed
 * in a given region for the purpose of using it in an AWS Route53 Alias record.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.apprunner.getHostedZoneId({});
 * const www = new aws.route53.Record("www", {
 *     zoneId: primary.zoneId,
 *     name: "example.com",
 *     type: aws.route53.RecordType.A,
 *     aliases: [{
 *         name: mainAwsApprunnerCustomDomainAssociation.dnsTarget,
 *         zoneId: main.then(main => main.id),
 *         evaluateTargetHealth: true,
 *     }],
 * });
 * ```
 */
export declare function getHostedZoneIdOutput(args?: GetHostedZoneIdOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetHostedZoneIdResult>;
/**
 * A collection of arguments for invoking getHostedZoneId.
 */
export interface GetHostedZoneIdOutputArgs {
    /**
     * Name of the Region whose AWS App Runner service HostedZoneId is desired. Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

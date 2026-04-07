import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the HostedZoneId of the AWS Elastic Load Balancing (ELB) in a given region for the purpose of using in an AWS Route53 Alias. Specify the ELB type (`network` or `application`) to return the relevant the associated HostedZoneId. Ref: [ELB service endpoints](https://docs.aws.amazon.com/general/latest/gr/elb.html#elb_region)
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.lb.getHostedZoneId({});
 * const www = new aws.route53.Record("www", {
 *     zoneId: primary.zoneId,
 *     name: "example.com",
 *     type: aws.route53.RecordType.A,
 *     aliases: [{
 *         name: mainAwsLb.dnsName,
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
     * Type of load balancer to create. Possible values are `application` or `network`. The default value is `application`.
     */
    loadBalancerType?: string;
    /**
     * Name of the Region whose AWS ELB HostedZoneId is desired. Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getHostedZoneId.
 */
export interface GetHostedZoneIdResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly loadBalancerType?: string;
    readonly region: string;
}
/**
 * Use this data source to get the HostedZoneId of the AWS Elastic Load Balancing (ELB) in a given region for the purpose of using in an AWS Route53 Alias. Specify the ELB type (`network` or `application`) to return the relevant the associated HostedZoneId. Ref: [ELB service endpoints](https://docs.aws.amazon.com/general/latest/gr/elb.html#elb_region)
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.lb.getHostedZoneId({});
 * const www = new aws.route53.Record("www", {
 *     zoneId: primary.zoneId,
 *     name: "example.com",
 *     type: aws.route53.RecordType.A,
 *     aliases: [{
 *         name: mainAwsLb.dnsName,
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
     * Type of load balancer to create. Possible values are `application` or `network`. The default value is `application`.
     */
    loadBalancerType?: pulumi.Input<string>;
    /**
     * Name of the Region whose AWS ELB HostedZoneId is desired. Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

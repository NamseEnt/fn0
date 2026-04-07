import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Route53 Hosted Zone. For managing Domain Name System Security Extensions (DNSSEC), see the `aws.route53.KeySigningKey` and `aws.route53.HostedZoneDnsSec` resources.
 *
 * ## Example Usage
 *
 * ### Public Zone
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const primary = new aws.route53.Zone("primary", {name: "example.com"});
 * ```
 *
 * ### Public Subdomain Zone
 *
 * For use in subdomains, note that you need to create a
 * `aws.route53.Record` of type `NS` as well as the subdomain
 * zone.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.route53.Zone("main", {name: "example.com"});
 * const dev = new aws.route53.Zone("dev", {
 *     name: "dev.example.com",
 *     tags: {
 *         Environment: "dev",
 *     },
 * });
 * const dev_ns = new aws.route53.Record("dev-ns", {
 *     zoneId: main.zoneId,
 *     name: "dev.example.com",
 *     type: aws.route53.RecordType.NS,
 *     ttl: 30,
 *     records: dev.nameServers,
 * });
 * ```
 *
 * ### Private Zone
 *
 * > **NOTE:** This provider provides both exclusive VPC associations defined in-line in this resource via `vpc` configuration blocks and a separate ` Zone VPC Association resource. At this time, you cannot use in-line VPC associations in conjunction with any  `aws.route53.ZoneAssociation`  resources with the same zone ID otherwise it will cause a perpetual difference in plan output. You can optionally use [ `ignoreChanges` ](https://www.pulumi.com/docs/intro/concepts/programming-model/#ignorechanges) to manage additional associations via the  `aws.route53.ZoneAssociation` resource.
 *
 * > **NOTE:** Private zones require at least one VPC association at all times.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const primary = new aws.ec2.Vpc("primary", {
 *     cidrBlock: "10.6.0.0/16",
 *     enableDnsHostnames: true,
 *     enableDnsSupport: true,
 * });
 * const secondary = new aws.ec2.Vpc("secondary", {
 *     cidrBlock: "10.7.0.0/16",
 *     enableDnsHostnames: true,
 *     enableDnsSupport: true,
 * });
 * const _private = new aws.route53.Zone("private", {
 *     name: "example.com",
 *     vpcs: [
 *         {
 *             vpcId: primary.id,
 *         },
 *         {
 *             vpcId: secondary.id,
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Zones using the zone `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/zone:Zone myzone Z1D633PJN98FT9
 * ```
 */
export declare class Zone extends pulumi.CustomResource {
    /**
     * Get an existing Zone resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ZoneState, opts?: pulumi.CustomResourceOptions): Zone;
    /**
     * Returns true if the given object is an instance of Zone.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Zone;
    /**
     * The Amazon Resource Name (ARN) of the Hosted Zone.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A comment for the hosted zone. Defaults to 'Managed by Pulumi'.
     */
    readonly comment: pulumi.Output<string>;
    /**
     * The ID of the reusable delegation set whose NS records you want to assign to the hosted zone. Conflicts with `vpc` as delegation sets can only be used for public zones.
     */
    readonly delegationSetId: pulumi.Output<string | undefined>;
    /**
     * Whether to destroy all records (possibly managed outside of this provider) in the zone when destroying the zone.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * This is the name of the hosted zone.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A list of name servers in associated (or default) delegation set.
     * Find more about delegation sets in [AWS docs](https://docs.aws.amazon.com/Route53/latest/APIReference/actions-on-reusable-delegation-sets.html).
     */
    readonly nameServers: pulumi.Output<string[]>;
    /**
     * The Route 53 name server that created the SOA record.
     */
    readonly primaryNameServer: pulumi.Output<string>;
    /**
     * A mapping of tags to assign to the zone. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Configuration block(s) specifying VPC(s) to associate with a private hosted zone. Conflicts with the `delegationSetId` argument in this resource and any `aws.route53.ZoneAssociation` resource specifying the same zone ID. Detailed below.
     */
    readonly vpcs: pulumi.Output<outputs.route53.ZoneVpc[] | undefined>;
    /**
     * The Hosted Zone ID. This can be referenced by zone records.
     */
    readonly zoneId: pulumi.Output<string>;
    /**
     * Create a Zone resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ZoneArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Zone resources.
 */
export interface ZoneState {
    /**
     * The Amazon Resource Name (ARN) of the Hosted Zone.
     */
    arn?: pulumi.Input<string>;
    /**
     * A comment for the hosted zone. Defaults to 'Managed by Pulumi'.
     */
    comment?: pulumi.Input<string>;
    /**
     * The ID of the reusable delegation set whose NS records you want to assign to the hosted zone. Conflicts with `vpc` as delegation sets can only be used for public zones.
     */
    delegationSetId?: pulumi.Input<string>;
    /**
     * Whether to destroy all records (possibly managed outside of this provider) in the zone when destroying the zone.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * This is the name of the hosted zone.
     */
    name?: pulumi.Input<string>;
    /**
     * A list of name servers in associated (or default) delegation set.
     * Find more about delegation sets in [AWS docs](https://docs.aws.amazon.com/Route53/latest/APIReference/actions-on-reusable-delegation-sets.html).
     */
    nameServers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Route 53 name server that created the SOA record.
     */
    primaryNameServer?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the zone. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block(s) specifying VPC(s) to associate with a private hosted zone. Conflicts with the `delegationSetId` argument in this resource and any `aws.route53.ZoneAssociation` resource specifying the same zone ID. Detailed below.
     */
    vpcs?: pulumi.Input<pulumi.Input<inputs.route53.ZoneVpc>[]>;
    /**
     * The Hosted Zone ID. This can be referenced by zone records.
     */
    zoneId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Zone resource.
 */
export interface ZoneArgs {
    /**
     * A comment for the hosted zone. Defaults to 'Managed by Pulumi'.
     */
    comment?: pulumi.Input<string>;
    /**
     * The ID of the reusable delegation set whose NS records you want to assign to the hosted zone. Conflicts with `vpc` as delegation sets can only be used for public zones.
     */
    delegationSetId?: pulumi.Input<string>;
    /**
     * Whether to destroy all records (possibly managed outside of this provider) in the zone when destroying the zone.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * This is the name of the hosted zone.
     */
    name?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the zone. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block(s) specifying VPC(s) to associate with a private hosted zone. Conflicts with the `delegationSetId` argument in this resource and any `aws.route53.ZoneAssociation` resource specifying the same zone ID. Detailed below.
     */
    vpcs?: pulumi.Input<pulumi.Input<inputs.route53.ZoneVpc>[]>;
}

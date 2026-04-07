import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Config Aggregate Authorization
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cfg.AggregateAuthorization("example", {
 *     accountId: "123456789012",
 *     authorizedAwsRegion: "eu-west-2",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Config aggregate authorizations using `account_id:authorized_aws_region`. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/aggregateAuthorization:AggregateAuthorization example 123456789012:us-east-1
 * ```
 */
export declare class AggregateAuthorization extends pulumi.CustomResource {
    /**
     * Get an existing AggregateAuthorization resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AggregateAuthorizationState, opts?: pulumi.CustomResourceOptions): AggregateAuthorization;
    /**
     * Returns true if the given object is an instance of AggregateAuthorization.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AggregateAuthorization;
    /**
     * Account ID.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The ARN of the authorization
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The region authorized to collect aggregated data.
     */
    readonly authorizedAwsRegion: pulumi.Output<string | undefined>;
    /**
     * The region authorized to collect aggregated data. Use `authorizedAwsRegion` instead.
     *
     * @deprecated region is deprecated. Use authorizedAwsRegion instead.
     */
    readonly region: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a AggregateAuthorization resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AggregateAuthorizationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AggregateAuthorization resources.
 */
export interface AggregateAuthorizationState {
    /**
     * Account ID.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The ARN of the authorization
     */
    arn?: pulumi.Input<string>;
    /**
     * The region authorized to collect aggregated data.
     */
    authorizedAwsRegion?: pulumi.Input<string>;
    /**
     * The region authorized to collect aggregated data. Use `authorizedAwsRegion` instead.
     *
     * @deprecated region is deprecated. Use authorizedAwsRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a AggregateAuthorization resource.
 */
export interface AggregateAuthorizationArgs {
    /**
     * Account ID.
     */
    accountId: pulumi.Input<string>;
    /**
     * The region authorized to collect aggregated data.
     */
    authorizedAwsRegion?: pulumi.Input<string>;
    /**
     * The region authorized to collect aggregated data. Use `authorizedAwsRegion` instead.
     *
     * @deprecated region is deprecated. Use authorizedAwsRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

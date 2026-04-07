import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the content and status of IP rules.
 *
 * > Deletion of this resource clears all IP restrictions from a QuickSight account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.IpRestriction("example", {
 *     enabled: true,
 *     ipRestrictionRuleMap: {
 *         "108.56.166.202/32": "Allow self",
 *     },
 *     vpcIdRestrictionRuleMap: {
 *         [exampleAwsVpc.id]: "Main VPC",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight IP restriction using the AWS account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/ipRestriction:IpRestriction example "012345678901"
 * ```
 */
export declare class IpRestriction extends pulumi.CustomResource {
    /**
     * Get an existing IpRestriction resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IpRestrictionState, opts?: pulumi.CustomResourceOptions): IpRestriction;
    /**
     * Returns true if the given object is an instance of IpRestriction.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IpRestriction;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Whether IP rules are turned on.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Map of allowed IPv4 CIDR ranges and descriptions.
     */
    readonly ipRestrictionRuleMap: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of allowed VPC endpoint IDs and descriptions.
     */
    readonly vpcEndpointIdRestrictionRuleMap: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of VPC IDs and descriptions. Traffic from all VPC endpoints that are present in the specified VPC is allowed.
     */
    readonly vpcIdRestrictionRuleMap: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a IpRestriction resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IpRestrictionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IpRestriction resources.
 */
export interface IpRestrictionState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Whether IP rules are turned on.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Map of allowed IPv4 CIDR ranges and descriptions.
     */
    ipRestrictionRuleMap?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of allowed VPC endpoint IDs and descriptions.
     */
    vpcEndpointIdRestrictionRuleMap?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of VPC IDs and descriptions. Traffic from all VPC endpoints that are present in the specified VPC is allowed.
     */
    vpcIdRestrictionRuleMap?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a IpRestriction resource.
 */
export interface IpRestrictionArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Whether IP rules are turned on.
     */
    enabled: pulumi.Input<boolean>;
    /**
     * Map of allowed IPv4 CIDR ranges and descriptions.
     */
    ipRestrictionRuleMap?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of allowed VPC endpoint IDs and descriptions.
     */
    vpcEndpointIdRestrictionRuleMap?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of VPC IDs and descriptions. Traffic from all VPC endpoints that are present in the specified VPC is allowed.
     */
    vpcIdRestrictionRuleMap?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

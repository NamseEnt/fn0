import * as pulumi from "@pulumi/pulumi";
/**
 * Enables AWS Shield Advanced for a specific AWS resource.
 * The resource can be an Amazon CloudFront distribution, Elastic Load Balancing load balancer, AWS Global Accelerator accelerator, Elastic IP Address, or an Amazon Route 53 hosted zone.
 *
 * ## Example Usage
 *
 * ### Create protection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.getAvailabilityZones({});
 * const current = aws.getRegion({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const example = new aws.ec2.Eip("example", {domain: "vpc"});
 * const exampleProtection = new aws.shield.Protection("example", {
 *     name: "example",
 *     resourceArn: pulumi.all([current, currentGetCallerIdentity, example.id]).apply(([current, currentGetCallerIdentity, id]) => `arn:aws:ec2:${current.region}:${currentGetCallerIdentity.accountId}:eip-allocation/${id}`),
 *     tags: {
 *         Environment: "Dev",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Shield protection resources using specifying their ID. For example:
 *
 * ```sh
 * $ pulumi import aws:shield/protection:Protection example ff9592dc-22f3-4e88-afa1-7b29fde9669a
 * ```
 */
export declare class Protection extends pulumi.CustomResource {
    /**
     * Get an existing Protection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProtectionState, opts?: pulumi.CustomResourceOptions): Protection;
    /**
     * Returns true if the given object is an instance of Protection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Protection;
    /**
     * The ARN of the Protection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A friendly name for the Protection you are creating.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ARN (Amazon Resource Name) of the resource to be protected.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Protection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProtectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Protection resources.
 */
export interface ProtectionState {
    /**
     * The ARN of the Protection.
     */
    arn?: pulumi.Input<string>;
    /**
     * A friendly name for the Protection you are creating.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN (Amazon Resource Name) of the resource to be protected.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Protection resource.
 */
export interface ProtectionArgs {
    /**
     * A friendly name for the Protection you are creating.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN (Amazon Resource Name) of the resource to be protected.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

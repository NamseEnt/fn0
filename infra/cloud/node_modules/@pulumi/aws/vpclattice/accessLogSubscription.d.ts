import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS VPC Lattice Service Network or Service Access log subscription.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.AccessLogSubscription("example", {
 *     resourceIdentifier: exampleAwsVpclatticeServiceNetwork.id,
 *     destinationArn: bucket.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Access Log Subscription using the access log subscription ID. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/accessLogSubscription:AccessLogSubscription example rft-8012925589
 * ```
 */
export declare class AccessLogSubscription extends pulumi.CustomResource {
    /**
     * Get an existing AccessLogSubscription resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccessLogSubscriptionState, opts?: pulumi.CustomResourceOptions): AccessLogSubscription;
    /**
     * Returns true if the given object is an instance of AccessLogSubscription.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccessLogSubscription;
    /**
     * Amazon Resource Name (ARN) of the access log subscription.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the log destination.
     */
    readonly destinationArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the service network or service.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service network or service. You must use the ARN if the resources specified in the operation are in different accounts.
     *
     * The following arguments are optional:
     */
    readonly resourceIdentifier: pulumi.Output<string>;
    /**
     * Type of log that monitors your Amazon VPC Lattice service networks. Valid values are: `SERVICE`, `RESOURCE`. Defaults to `SERVICE`.
     */
    readonly serviceNetworkLogType: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a AccessLogSubscription resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccessLogSubscriptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccessLogSubscription resources.
 */
export interface AccessLogSubscriptionState {
    /**
     * Amazon Resource Name (ARN) of the access log subscription.
     */
    arn?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the log destination.
     */
    destinationArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the service network or service.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service network or service. You must use the ARN if the resources specified in the operation are in different accounts.
     *
     * The following arguments are optional:
     */
    resourceIdentifier?: pulumi.Input<string>;
    /**
     * Type of log that monitors your Amazon VPC Lattice service networks. Valid values are: `SERVICE`, `RESOURCE`. Defaults to `SERVICE`.
     */
    serviceNetworkLogType?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a AccessLogSubscription resource.
 */
export interface AccessLogSubscriptionArgs {
    /**
     * Amazon Resource Name (ARN) of the log destination.
     */
    destinationArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service network or service. You must use the ARN if the resources specified in the operation are in different accounts.
     *
     * The following arguments are optional:
     */
    resourceIdentifier: pulumi.Input<string>;
    /**
     * Type of log that monitors your Amazon VPC Lattice service networks. Valid values are: `SERVICE`, `RESOURCE`. Defaults to `SERVICE`.
     */
    serviceNetworkLogType?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

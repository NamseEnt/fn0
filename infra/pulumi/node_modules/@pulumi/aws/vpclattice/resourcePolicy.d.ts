import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS VPC Lattice Resource Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const example = new aws.vpclattice.ServiceNetwork("example", {name: "example-vpclattice-service-network"});
 * const exampleResourcePolicy = new aws.vpclattice.ResourcePolicy("example", {
 *     resourceArn: example.arn,
 *     policy: pulumi.jsonStringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Sid: "test-pol-principals-6",
 *             Effect: "Allow",
 *             Principal: {
 *                 AWS: Promise.all([currentGetPartition, current]).then(([currentGetPartition, current]) => `arn:${currentGetPartition.partition}:iam::${current.accountId}:root`),
 *             },
 *             Action: [
 *                 "vpc-lattice:CreateServiceNetworkVpcAssociation",
 *                 "vpc-lattice:CreateServiceNetworkServiceAssociation",
 *                 "vpc-lattice:GetServiceNetwork",
 *             ],
 *             Resource: example.arn,
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Resource Policy using the `resource_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/resourcePolicy:ResourcePolicy example rft-8012925589
 * ```
 */
export declare class ResourcePolicy extends pulumi.CustomResource {
    /**
     * Get an existing ResourcePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourcePolicyState, opts?: pulumi.CustomResourceOptions): ResourcePolicy;
    /**
     * Returns true if the given object is an instance of ResourcePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourcePolicy;
    /**
     * An IAM policy. The policy string in JSON must not contain newlines or blank lines.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID or Amazon Resource Name (ARN) of the service network or service for which the policy is created.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a ResourcePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourcePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourcePolicy resources.
 */
export interface ResourcePolicyState {
    /**
     * An IAM policy. The policy string in JSON must not contain newlines or blank lines.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Name (ARN) of the service network or service for which the policy is created.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourcePolicy resource.
 */
export interface ResourcePolicyArgs {
    /**
     * An IAM policy. The policy string in JSON must not contain newlines or blank lines.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Name (ARN) of the service network or service for which the policy is created.
     */
    resourceArn: pulumi.Input<string>;
}

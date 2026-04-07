import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
/**
 * Provides an Elastic Container Registry Policy.
 *
 * > **NOTE on ECR Registry Policies:** While the AWS Management Console interface may suggest the ability to define multiple policies by creating multiple statements, ECR registry policies are effectively managed as singular entities at the regional level by the AWS APIs. Therefore, the `aws.ecr.RegistryPolicy` resource should be configured only once per region with all necessary statements defined in the same policy. Attempting to define multiple `aws.ecr.RegistryPolicy` resources may result in perpetual differences, with one policy overriding another.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetRegion = aws.getRegion({});
 * const currentGetPartition = aws.getPartition({});
 * const example = new aws.ecr.RegistryPolicy("example", {policy: JSON.stringify({
 *     Version: "2012-10-17",
 *     Statement: [{
 *         Sid: "testpolicy",
 *         Effect: "Allow",
 *         Principal: {
 *             AWS: Promise.all([currentGetPartition, current]).then(([currentGetPartition, current]) => `arn:${currentGetPartition.partition}:iam::${current.accountId}:root`),
 *         },
 *         Action: ["ecr:ReplicateImage"],
 *         Resource: [Promise.all([currentGetPartition, currentGetRegion, current]).then(([currentGetPartition, currentGetRegion, current]) => `arn:${currentGetPartition.partition}:ecr:${currentGetRegion.region}:${current.accountId}:repository/*`)],
 *     }],
 * })});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECR Registry Policy using the registry id. For example:
 *
 * ```sh
 * $ pulumi import aws:ecr/registryPolicy:RegistryPolicy example 123456789012
 * ```
 */
export declare class RegistryPolicy extends pulumi.CustomResource {
    /**
     * Get an existing RegistryPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RegistryPolicyState, opts?: pulumi.CustomResourceOptions): RegistryPolicy;
    /**
     * Returns true if the given object is an instance of RegistryPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RegistryPolicy;
    /**
     * The policy document. This is a JSON formatted string.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID where the registry was created.
     */
    readonly registryId: pulumi.Output<string>;
    /**
     * Create a RegistryPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RegistryPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RegistryPolicy resources.
 */
export interface RegistryPolicyState {
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy?: pulumi.Input<string | inputs.ecr.PolicyDocument>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID where the registry was created.
     */
    registryId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RegistryPolicy resource.
 */
export interface RegistryPolicyArgs {
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy: pulumi.Input<string | inputs.ecr.PolicyDocument>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

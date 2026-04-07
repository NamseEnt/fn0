import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a new Amazon Redshift Serverless Resource Policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshiftserverless.ResourcePolicy("example", {
 *     resourceArn: exampleAwsRedshiftserverlessSnapshot.arn,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Principal: {
 *                 AWS: ["12345678901"],
 *             },
 *             Action: ["redshift-serverless:RestoreFromSnapshot"],
 *             Sid: "",
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Serverless Resource Policies using the `resource_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshiftserverless/resourcePolicy:ResourcePolicy example example
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
     * The policy to create or update. For example, the following policy grants a user authorization to restore a snapshot.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the account to create or update a resource policy for.
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
     * The policy to create or update. For example, the following policy grants a user authorization to restore a snapshot.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the account to create or update a resource policy for.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourcePolicy resource.
 */
export interface ResourcePolicyArgs {
    /**
     * The policy to create or update. For example, the following policy grants a user authorization to restore a snapshot.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the account to create or update a resource policy for.
     */
    resourceArn: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const exampleContainer = new aws.mediastore.Container("example", {name: "example"});
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         sid: "MediaStoreFullAccess",
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: [currentGetCallerIdentity.then(currentGetCallerIdentity => `arn:aws:iam::${currentGetCallerIdentity.accountId}:root`)],
 *         }],
 *         actions: ["mediastore:*"],
 *         resources: [pulumi.all([current, currentGetCallerIdentity, exampleContainer.name]).apply(([current, currentGetCallerIdentity, name]) => `arn:aws:mediastore:${current.region}:${currentGetCallerIdentity.accountId}:container/${name}/*`)],
 *         conditions: [{
 *             test: "Bool",
 *             variable: "aws:SecureTransport",
 *             values: ["true"],
 *         }],
 *     }],
 * });
 * const exampleContainerPolicy = new aws.mediastore.ContainerPolicy("example", {
 *     containerName: exampleContainer.name,
 *     policy: example.apply(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MediaStore Container Policy using the MediaStore Container Name. For example:
 *
 * ```sh
 * $ pulumi import aws:mediastore/containerPolicy:ContainerPolicy example example
 * ```
 */
export declare class ContainerPolicy extends pulumi.CustomResource {
    /**
     * Get an existing ContainerPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContainerPolicyState, opts?: pulumi.CustomResourceOptions): ContainerPolicy;
    /**
     * Returns true if the given object is an instance of ContainerPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContainerPolicy;
    /**
     * The name of the container.
     */
    readonly containerName: pulumi.Output<string>;
    /**
     * The contents of the policy.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ContainerPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContainerPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContainerPolicy resources.
 */
export interface ContainerPolicyState {
    /**
     * The name of the container.
     */
    containerName?: pulumi.Input<string>;
    /**
     * The contents of the policy.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ContainerPolicy resource.
 */
export interface ContainerPolicyArgs {
    /**
     * The name of the container.
     */
    containerName: pulumi.Input<string>;
    /**
     * The contents of the policy.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

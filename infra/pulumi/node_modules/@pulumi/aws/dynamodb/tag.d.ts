import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an individual DynamoDB resource tag. This resource should only be used in cases where DynamoDB resources are created outside the provider (e.g., Table replicas in other regions).
 *
 * > **NOTE:** This tagging resource should not be combined with the resource for managing the parent resource. For example, using `aws.dynamodb.Table` and `aws.dynamodb.Tag` to manage tags of the same DynamoDB Table in the same region will cause a perpetual difference where the `awsDynamodbCluster` resource will try to remove the tag being added by the `aws.dynamodb.Tag` resource.
 *
 * > **NOTE:** This tagging resource does not use the provider `ignoreTags` configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const replica = aws.getRegion({});
 * const current = aws.getRegion({});
 * const example = new aws.dynamodb.Table("example", {replicas: [{
 *     regionName: replica.then(replica => replica.name),
 * }]});
 * const test = new aws.dynamodb.Tag("test", {
 *     resourceArn: pulumi.all([example.arn, current, replica]).apply(([arn, current, replica]) => std.replaceOutput({
 *         text: arn,
 *         search: current.region,
 *         replace: replica.name,
 *     })).apply(invoke => invoke.result),
 *     key: "testkey",
 *     value: "testvalue",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_dynamodb_tag` using the DynamoDB resource identifier and key, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:dynamodb/tag:Tag example arn:aws:dynamodb:us-east-1:123456789012:table/example,Name
 * ```
 */
export declare class Tag extends pulumi.CustomResource {
    /**
     * Get an existing Tag resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TagState, opts?: pulumi.CustomResourceOptions): Tag;
    /**
     * Returns true if the given object is an instance of Tag.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Tag;
    /**
     * Tag name.
     */
    readonly key: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the DynamoDB resource to tag.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Tag value.
     */
    readonly value: pulumi.Output<string>;
    /**
     * Create a Tag resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TagArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Tag resources.
 */
export interface TagState {
    /**
     * Tag name.
     */
    key?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the DynamoDB resource to tag.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Tag value.
     */
    value?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Tag resource.
 */
export interface TagArgs {
    /**
     * Tag name.
     */
    key: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the DynamoDB resource to tag.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * Tag value.
     */
    value: pulumi.Input<string>;
}

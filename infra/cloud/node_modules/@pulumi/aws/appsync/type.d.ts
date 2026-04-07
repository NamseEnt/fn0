import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppSync Type.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appsync.GraphQLApi("example", {
 *     authenticationType: "API_KEY",
 *     name: "example",
 * });
 * const exampleType = new aws.appsync.Type("example", {
 *     apiId: example.id,
 *     format: "SDL",
 *     definition: `type Mutation
 *
 * {
 * putPost(id: ID!,title: String! ): Post
 *
 * }
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Appsync Types using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:appsync/type:Type example api-id:format:name
 * ```
 */
export declare class Type extends pulumi.CustomResource {
    /**
     * Get an existing Type resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TypeState, opts?: pulumi.CustomResourceOptions): Type;
    /**
     * Returns true if the given object is an instance of Type.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Type;
    /**
     * GraphQL API ID.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * The ARN of the type.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The type definition.
     */
    readonly definition: pulumi.Output<string>;
    /**
     * The type description.
     */
    readonly description: pulumi.Output<string>;
    /**
     * The type format: `SDL` or `JSON`.
     */
    readonly format: pulumi.Output<string>;
    /**
     * The type name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Type resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TypeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Type resources.
 */
export interface TypeState {
    /**
     * GraphQL API ID.
     */
    apiId?: pulumi.Input<string>;
    /**
     * The ARN of the type.
     */
    arn?: pulumi.Input<string>;
    /**
     * The type definition.
     */
    definition?: pulumi.Input<string>;
    /**
     * The type description.
     */
    description?: pulumi.Input<string>;
    /**
     * The type format: `SDL` or `JSON`.
     */
    format?: pulumi.Input<string>;
    /**
     * The type name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Type resource.
 */
export interface TypeArgs {
    /**
     * GraphQL API ID.
     */
    apiId: pulumi.Input<string>;
    /**
     * The type definition.
     */
    definition: pulumi.Input<string>;
    /**
     * The type format: `SDL` or `JSON`.
     */
    format: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

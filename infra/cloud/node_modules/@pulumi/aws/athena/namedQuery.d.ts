import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Athena Named Query resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const hoge = new aws.s3.Bucket("hoge", {bucket: "tf-test"});
 * const test = new aws.kms.Key("test", {
 *     deletionWindowInDays: 7,
 *     description: "Athena KMS Key",
 * });
 * const testWorkgroup = new aws.athena.Workgroup("test", {
 *     name: "example",
 *     configuration: {
 *         resultConfiguration: {
 *             encryptionConfiguration: {
 *                 encryptionOption: "SSE_KMS",
 *                 kmsKeyArn: test.arn,
 *             },
 *         },
 *     },
 * });
 * const hogeDatabase = new aws.athena.Database("hoge", {
 *     name: "users",
 *     bucket: hoge.id,
 * });
 * const foo = new aws.athena.NamedQuery("foo", {
 *     name: "bar",
 *     workgroup: testWorkgroup.id,
 *     database: hogeDatabase.name,
 *     query: pulumi.interpolate`SELECT * FROM ${hogeDatabase.name} limit 10;`,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Athena Named Query using the query ID. For example:
 *
 * ```sh
 * $ pulumi import aws:athena/namedQuery:NamedQuery example 0123456789
 * ```
 */
export declare class NamedQuery extends pulumi.CustomResource {
    /**
     * Get an existing NamedQuery resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NamedQueryState, opts?: pulumi.CustomResourceOptions): NamedQuery;
    /**
     * Returns true if the given object is an instance of NamedQuery.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NamedQuery;
    /**
     * Database to which the query belongs.
     */
    readonly database: pulumi.Output<string>;
    /**
     * Brief explanation of the query. Maximum length of 1024.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Plain language name for the query. Maximum length of 128.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Text of the query itself. In other words, all query statements. Maximum length of 262144.
     */
    readonly query: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Workgroup to which the query belongs. Defaults to `primary`
     */
    readonly workgroup: pulumi.Output<string | undefined>;
    /**
     * Create a NamedQuery resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NamedQueryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NamedQuery resources.
 */
export interface NamedQueryState {
    /**
     * Database to which the query belongs.
     */
    database?: pulumi.Input<string>;
    /**
     * Brief explanation of the query. Maximum length of 1024.
     */
    description?: pulumi.Input<string>;
    /**
     * Plain language name for the query. Maximum length of 128.
     */
    name?: pulumi.Input<string>;
    /**
     * Text of the query itself. In other words, all query statements. Maximum length of 262144.
     */
    query?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Workgroup to which the query belongs. Defaults to `primary`
     */
    workgroup?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NamedQuery resource.
 */
export interface NamedQueryArgs {
    /**
     * Database to which the query belongs.
     */
    database: pulumi.Input<string>;
    /**
     * Brief explanation of the query. Maximum length of 1024.
     */
    description?: pulumi.Input<string>;
    /**
     * Plain language name for the query. Maximum length of 128.
     */
    name?: pulumi.Input<string>;
    /**
     * Text of the query itself. In other words, all query statements. Maximum length of 262144.
     */
    query: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Workgroup to which the query belongs. Defaults to `primary`
     */
    workgroup?: pulumi.Input<string>;
}

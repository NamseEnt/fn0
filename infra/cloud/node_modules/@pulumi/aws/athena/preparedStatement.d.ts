import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an Athena Prepared Statement.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.s3.Bucket("test", {
 *     bucket: "tf-test",
 *     forceDestroy: true,
 * });
 * const testWorkgroup = new aws.athena.Workgroup("test", {name: "tf-test"});
 * const testDatabase = new aws.athena.Database("test", {
 *     name: "example",
 *     bucket: test.bucket,
 * });
 * const testPreparedStatement = new aws.athena.PreparedStatement("test", {
 *     name: "tf_test",
 *     queryStatement: pulumi.interpolate`SELECT * FROM ${testDatabase.name} WHERE x = ?`,
 *     workgroup: testWorkgroup.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Athena Prepared Statement using the `WORKGROUP-NAME/STATEMENT-NAME`. For example:
 *
 * ```sh
 * $ pulumi import aws:athena/preparedStatement:PreparedStatement example 12345abcde/example
 * ```
 */
export declare class PreparedStatement extends pulumi.CustomResource {
    /**
     * Get an existing PreparedStatement resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PreparedStatementState, opts?: pulumi.CustomResourceOptions): PreparedStatement;
    /**
     * Returns true if the given object is an instance of PreparedStatement.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PreparedStatement;
    /**
     * Brief explanation of prepared statement. Maximum length of 1024.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the prepared statement. Maximum length of 256.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The query string for the prepared statement.
     */
    readonly queryStatement: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the workgroup to which the prepared statement belongs.
     */
    readonly workgroup: pulumi.Output<string>;
    /**
     * Create a PreparedStatement resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PreparedStatementArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PreparedStatement resources.
 */
export interface PreparedStatementState {
    /**
     * Brief explanation of prepared statement. Maximum length of 1024.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the prepared statement. Maximum length of 256.
     */
    name?: pulumi.Input<string>;
    /**
     * The query string for the prepared statement.
     */
    queryStatement?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the workgroup to which the prepared statement belongs.
     */
    workgroup?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PreparedStatement resource.
 */
export interface PreparedStatementArgs {
    /**
     * Brief explanation of prepared statement. Maximum length of 1024.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the prepared statement. Maximum length of 256.
     */
    name?: pulumi.Input<string>;
    /**
     * The query string for the prepared statement.
     */
    queryStatement: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the workgroup to which the prepared statement belongs.
     */
    workgroup: pulumi.Input<string>;
}

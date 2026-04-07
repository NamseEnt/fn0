import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an attachment between one or more existing LF-tags and an existing Lake Formation resource.
 *
 * ## Example Usage
 *
 * ### Database Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.LfTag("example", {
 *     key: "right",
 *     values: [
 *         "abbey",
 *         "village",
 *         "luffield",
 *         "woodcote",
 *         "copse",
 *         "chapel",
 *         "stowe",
 *         "club",
 *     ],
 * });
 * const exampleResourceLfTags = new aws.lakeformation.ResourceLfTags("example", {
 *     database: {
 *         name: exampleAwsGlueCatalogDatabase.name,
 *     },
 *     lfTags: [{
 *         key: example.key,
 *         value: "stowe",
 *     }],
 * });
 * ```
 *
 * ### Multiple Tags Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.LfTag("example", {
 *     key: "right",
 *     values: [
 *         "abbey",
 *         "village",
 *         "luffield",
 *         "woodcote",
 *         "copse",
 *         "chapel",
 *         "stowe",
 *         "club",
 *     ],
 * });
 * const example2 = new aws.lakeformation.LfTag("example2", {
 *     key: "left",
 *     values: [
 *         "farm",
 *         "theloop",
 *         "aintree",
 *         "brooklands",
 *         "maggotts",
 *         "becketts",
 *         "vale",
 *     ],
 * });
 * const exampleResourceLfTags = new aws.lakeformation.ResourceLfTags("example", {
 *     database: {
 *         name: exampleAwsGlueCatalogDatabase.name,
 *     },
 *     lfTags: [
 *         {
 *             key: "right",
 *             value: "luffield",
 *         },
 *         {
 *             key: "left",
 *             value: "aintree",
 *         },
 *     ],
 * });
 * ```
 */
export declare class ResourceLfTags extends pulumi.CustomResource {
    /**
     * Get an existing ResourceLfTags resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceLfTagsState, opts?: pulumi.CustomResourceOptions): ResourceLfTags;
    /**
     * Returns true if the given object is an instance of ResourceLfTags.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceLfTags;
    /**
     * Identifier for the Data Catalog. By default, the account ID. The Data Catalog is the persistent metadata store. It contains database definitions, table definitions, and other control information to manage your Lake Formation environment.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * Configuration block for a database resource. See below.
     */
    readonly database: pulumi.Output<outputs.lakeformation.ResourceLfTagsDatabase>;
    /**
     * Set of LF-tags to attach to the resource. See below.
     *
     * Exactly one of the following is required:
     */
    readonly lfTags: pulumi.Output<outputs.lakeformation.ResourceLfTagsLfTag[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for a table resource. See below.
     */
    readonly table: pulumi.Output<outputs.lakeformation.ResourceLfTagsTable>;
    /**
     * Configuration block for a table with columns resource. See below.
     *
     * The following arguments are optional:
     */
    readonly tableWithColumns: pulumi.Output<outputs.lakeformation.ResourceLfTagsTableWithColumns>;
    /**
     * Create a ResourceLfTags resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceLfTagsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceLfTags resources.
 */
export interface ResourceLfTagsState {
    /**
     * Identifier for the Data Catalog. By default, the account ID. The Data Catalog is the persistent metadata store. It contains database definitions, table definitions, and other control information to manage your Lake Formation environment.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Configuration block for a database resource. See below.
     */
    database?: pulumi.Input<inputs.lakeformation.ResourceLfTagsDatabase>;
    /**
     * Set of LF-tags to attach to the resource. See below.
     *
     * Exactly one of the following is required:
     */
    lfTags?: pulumi.Input<pulumi.Input<inputs.lakeformation.ResourceLfTagsLfTag>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for a table resource. See below.
     */
    table?: pulumi.Input<inputs.lakeformation.ResourceLfTagsTable>;
    /**
     * Configuration block for a table with columns resource. See below.
     *
     * The following arguments are optional:
     */
    tableWithColumns?: pulumi.Input<inputs.lakeformation.ResourceLfTagsTableWithColumns>;
}
/**
 * The set of arguments for constructing a ResourceLfTags resource.
 */
export interface ResourceLfTagsArgs {
    /**
     * Identifier for the Data Catalog. By default, the account ID. The Data Catalog is the persistent metadata store. It contains database definitions, table definitions, and other control information to manage your Lake Formation environment.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Configuration block for a database resource. See below.
     */
    database?: pulumi.Input<inputs.lakeformation.ResourceLfTagsDatabase>;
    /**
     * Set of LF-tags to attach to the resource. See below.
     *
     * Exactly one of the following is required:
     */
    lfTags: pulumi.Input<pulumi.Input<inputs.lakeformation.ResourceLfTagsLfTag>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for a table resource. See below.
     */
    table?: pulumi.Input<inputs.lakeformation.ResourceLfTagsTable>;
    /**
     * Configuration block for a table with columns resource. See below.
     *
     * The following arguments are optional:
     */
    tableWithColumns?: pulumi.Input<inputs.lakeformation.ResourceLfTagsTableWithColumns>;
}

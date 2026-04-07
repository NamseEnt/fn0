import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Lake Formation Resource LF Tag.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.ResourceLfTag("example", {
 *     database: {
 *         name: exampleAwsGlueCatalogDatabase.name,
 *     },
 *     lfTag: {
 *         key: exampleAwsLakeformationLfTag.key,
 *         value: "stowe",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * You cannot import this resource.
 */
export declare class ResourceLfTag extends pulumi.CustomResource {
    /**
     * Get an existing ResourceLfTag resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceLfTagState, opts?: pulumi.CustomResourceOptions): ResourceLfTag;
    /**
     * Returns true if the given object is an instance of ResourceLfTag.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceLfTag;
    /**
     * Identifier for the Data Catalog. By default, the account ID. The Data Catalog is the persistent metadata store. It contains database definitions, table definitions, and other control information to manage your Lake Formation environment.
     */
    readonly catalogId: pulumi.Output<string | undefined>;
    /**
     * Configuration block for a database resource. See Database for more details.
     */
    readonly database: pulumi.Output<outputs.lakeformation.ResourceLfTagDatabase | undefined>;
    /**
     * Set of LF-tags to attach to the resource. See LF Tag for more details.
     *
     * Exactly one of the following is required:
     */
    readonly lfTag: pulumi.Output<outputs.lakeformation.ResourceLfTagLfTag | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for a table resource. See Table for more details.
     */
    readonly table: pulumi.Output<outputs.lakeformation.ResourceLfTagTable | undefined>;
    /**
     * Configuration block for a table with columns resource. See Table With Columns for more details.
     *
     * The following arguments are optional:
     */
    readonly tableWithColumns: pulumi.Output<outputs.lakeformation.ResourceLfTagTableWithColumns | undefined>;
    readonly timeouts: pulumi.Output<outputs.lakeformation.ResourceLfTagTimeouts | undefined>;
    /**
     * Create a ResourceLfTag resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ResourceLfTagArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceLfTag resources.
 */
export interface ResourceLfTagState {
    /**
     * Identifier for the Data Catalog. By default, the account ID. The Data Catalog is the persistent metadata store. It contains database definitions, table definitions, and other control information to manage your Lake Formation environment.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Configuration block for a database resource. See Database for more details.
     */
    database?: pulumi.Input<inputs.lakeformation.ResourceLfTagDatabase>;
    /**
     * Set of LF-tags to attach to the resource. See LF Tag for more details.
     *
     * Exactly one of the following is required:
     */
    lfTag?: pulumi.Input<inputs.lakeformation.ResourceLfTagLfTag>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for a table resource. See Table for more details.
     */
    table?: pulumi.Input<inputs.lakeformation.ResourceLfTagTable>;
    /**
     * Configuration block for a table with columns resource. See Table With Columns for more details.
     *
     * The following arguments are optional:
     */
    tableWithColumns?: pulumi.Input<inputs.lakeformation.ResourceLfTagTableWithColumns>;
    timeouts?: pulumi.Input<inputs.lakeformation.ResourceLfTagTimeouts>;
}
/**
 * The set of arguments for constructing a ResourceLfTag resource.
 */
export interface ResourceLfTagArgs {
    /**
     * Identifier for the Data Catalog. By default, the account ID. The Data Catalog is the persistent metadata store. It contains database definitions, table definitions, and other control information to manage your Lake Formation environment.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Configuration block for a database resource. See Database for more details.
     */
    database?: pulumi.Input<inputs.lakeformation.ResourceLfTagDatabase>;
    /**
     * Set of LF-tags to attach to the resource. See LF Tag for more details.
     *
     * Exactly one of the following is required:
     */
    lfTag?: pulumi.Input<inputs.lakeformation.ResourceLfTagLfTag>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for a table resource. See Table for more details.
     */
    table?: pulumi.Input<inputs.lakeformation.ResourceLfTagTable>;
    /**
     * Configuration block for a table with columns resource. See Table With Columns for more details.
     *
     * The following arguments are optional:
     */
    tableWithColumns?: pulumi.Input<inputs.lakeformation.ResourceLfTagTableWithColumns>;
    timeouts?: pulumi.Input<inputs.lakeformation.ResourceLfTagTimeouts>;
}

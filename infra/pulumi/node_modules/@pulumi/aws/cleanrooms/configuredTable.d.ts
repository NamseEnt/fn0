import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a AWS Clean Rooms configured table. Configured tables are used to represent references to existing tables in the AWS Glue Data Catalog.
 *
 * ## Example Usage
 *
 * ### Configured table with tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testConfiguredTable = new aws.cleanrooms.ConfiguredTable("test_configured_table", {
 *     name: "pulumi-example-table",
 *     description: "I made this table with Pulumi!",
 *     analysisMethod: "DIRECT_QUERY",
 *     allowedColumns: [
 *         "column1",
 *         "column2",
 *         "column3",
 *     ],
 *     tableReference: {
 *         databaseName: "example_database",
 *         tableName: "example_table",
 *     },
 *     tags: {
 *         Project: "Pulumi",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `id` - (String) ID of the cleanrooms configured table.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import `aws_cleanrooms_configured_table` using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_cleanrooms_configured_table.table 1234abcd-12ab-34cd-56ef-1234567890ab
 */
export declare class ConfiguredTable extends pulumi.CustomResource {
    /**
     * Get an existing ConfiguredTable resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfiguredTableState, opts?: pulumi.CustomResourceOptions): ConfiguredTable;
    /**
     * Returns true if the given object is an instance of ConfiguredTable.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConfiguredTable;
    /**
     * The columns of the references table which will be included in the configured table.
     */
    readonly allowedColumns: pulumi.Output<string[]>;
    /**
     * The analysis method for the configured table. The only valid value is currently `DIRECT_QUERY`.
     */
    readonly analysisMethod: pulumi.Output<string>;
    /**
     * The ARN of the configured table.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date and time the configured table was created.
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * A description for the configured table.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the configured table.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A reference to the AWS Glue table which will be used to create the configured table.
     * * `table_reference.database_name` - (Required - Forces new resource) - The name of the AWS Glue database which contains the table.
     * * `table_reference.table_name` - (Required - Forces new resource) - The name of the AWS Glue table which will be used to create the configured table.
     */
    readonly tableReference: pulumi.Output<outputs.cleanrooms.ConfiguredTableTableReference>;
    /**
     * Key value pairs which tag the configured table.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The date and time the configured table was last updated.
     */
    readonly updateTime: pulumi.Output<string>;
    /**
     * Create a ConfiguredTable resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConfiguredTableArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConfiguredTable resources.
 */
export interface ConfiguredTableState {
    /**
     * The columns of the references table which will be included in the configured table.
     */
    allowedColumns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The analysis method for the configured table. The only valid value is currently `DIRECT_QUERY`.
     */
    analysisMethod?: pulumi.Input<string>;
    /**
     * The ARN of the configured table.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date and time the configured table was created.
     */
    createTime?: pulumi.Input<string>;
    /**
     * A description for the configured table.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the configured table.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A reference to the AWS Glue table which will be used to create the configured table.
     * * `table_reference.database_name` - (Required - Forces new resource) - The name of the AWS Glue database which contains the table.
     * * `table_reference.table_name` - (Required - Forces new resource) - The name of the AWS Glue table which will be used to create the configured table.
     */
    tableReference?: pulumi.Input<inputs.cleanrooms.ConfiguredTableTableReference>;
    /**
     * Key value pairs which tag the configured table.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The date and time the configured table was last updated.
     */
    updateTime?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConfiguredTable resource.
 */
export interface ConfiguredTableArgs {
    /**
     * The columns of the references table which will be included in the configured table.
     */
    allowedColumns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The analysis method for the configured table. The only valid value is currently `DIRECT_QUERY`.
     */
    analysisMethod: pulumi.Input<string>;
    /**
     * A description for the configured table.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the configured table.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A reference to the AWS Glue table which will be used to create the configured table.
     * * `table_reference.database_name` - (Required - Forces new resource) - The name of the AWS Glue database which contains the table.
     * * `table_reference.table_name` - (Required - Forces new resource) - The name of the AWS Glue table which will be used to create the configured table.
     */
    tableReference: pulumi.Input<inputs.cleanrooms.ConfiguredTableTableReference>;
    /**
     * Key value pairs which tag the configured table.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

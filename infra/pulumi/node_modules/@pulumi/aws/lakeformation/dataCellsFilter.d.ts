import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Lake Formation Data Cells Filter.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.DataCellsFilter("example", {tableData: {
 *     databaseName: test.name,
 *     name: "example",
 *     tableCatalogId: current.accountId,
 *     tableName: testAwsGlueCatalogTable.name,
 *     columnNames: ["my_column"],
 *     rowFilter: {
 *         filterExpression: "my_column='example'",
 *     },
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lake Formation Data Cells Filter using the `database_name`, `name`, `table_catalog_id`, and `table_name` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:lakeformation/dataCellsFilter:DataCellsFilter example database_name,name,table_catalog_id,table_name
 * ```
 */
export declare class DataCellsFilter extends pulumi.CustomResource {
    /**
     * Get an existing DataCellsFilter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataCellsFilterState, opts?: pulumi.CustomResourceOptions): DataCellsFilter;
    /**
     * Returns true if the given object is an instance of DataCellsFilter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataCellsFilter;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Information about the data cells filter. See Table Data below for details.
     */
    readonly tableData: pulumi.Output<outputs.lakeformation.DataCellsFilterTableData | undefined>;
    readonly timeouts: pulumi.Output<outputs.lakeformation.DataCellsFilterTimeouts | undefined>;
    /**
     * Create a DataCellsFilter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DataCellsFilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataCellsFilter resources.
 */
export interface DataCellsFilterState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Information about the data cells filter. See Table Data below for details.
     */
    tableData?: pulumi.Input<inputs.lakeformation.DataCellsFilterTableData>;
    timeouts?: pulumi.Input<inputs.lakeformation.DataCellsFilterTimeouts>;
}
/**
 * The set of arguments for constructing a DataCellsFilter resource.
 */
export interface DataCellsFilterArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Information about the data cells filter. See Table Data below for details.
     */
    tableData?: pulumi.Input<inputs.lakeformation.DataCellsFilterTableData>;
    timeouts?: pulumi.Input<inputs.lakeformation.DataCellsFilterTimeouts>;
}

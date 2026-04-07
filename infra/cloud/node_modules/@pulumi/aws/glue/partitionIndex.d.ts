import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogDatabase("example", {name: "example"});
 * const exampleCatalogTable = new aws.glue.CatalogTable("example", {
 *     name: "example",
 *     databaseName: example.name,
 *     owner: "my_owner",
 *     retention: 1,
 *     tableType: "VIRTUAL_VIEW",
 *     viewExpandedText: "view_expanded_text_1",
 *     viewOriginalText: "view_original_text_1",
 *     storageDescriptor: {
 *         bucketColumns: ["bucket_column_1"],
 *         compressed: false,
 *         inputFormat: "SequenceFileInputFormat",
 *         location: "my_location",
 *         numberOfBuckets: 1,
 *         outputFormat: "SequenceFileInputFormat",
 *         storedAsSubDirectories: false,
 *         parameters: {
 *             param1: "param1_val",
 *         },
 *         columns: [
 *             {
 *                 name: "my_column_1",
 *                 type: "int",
 *                 comment: "my_column1_comment",
 *             },
 *             {
 *                 name: "my_column_2",
 *                 type: "string",
 *                 comment: "my_column2_comment",
 *             },
 *         ],
 *         serDeInfo: {
 *             name: "ser_de_name",
 *             parameters: {
 *                 param1: "param_val_1",
 *             },
 *             serializationLibrary: "org.apache.hadoop.hive.serde2.columnar.ColumnarSerDe",
 *         },
 *         sortColumns: [{
 *             column: "my_column_1",
 *             sortOrder: 1,
 *         }],
 *         skewedInfo: {
 *             skewedColumnNames: ["my_column_1"],
 *             skewedColumnValueLocationMaps: {
 *                 my_column_1: "my_column_1_val_loc_map",
 *             },
 *             skewedColumnValues: ["skewed_val_1"],
 *         },
 *     },
 *     partitionKeys: [
 *         {
 *             name: "my_column_1",
 *             type: "int",
 *             comment: "my_column_1_comment",
 *         },
 *         {
 *             name: "my_column_2",
 *             type: "string",
 *             comment: "my_column_2_comment",
 *         },
 *     ],
 *     parameters: {
 *         param1: "param1_val",
 *     },
 * });
 * const examplePartitionIndex = new aws.glue.PartitionIndex("example", {
 *     databaseName: example.name,
 *     tableName: exampleCatalogTable.name,
 *     partitionIndex: {
 *         indexName: "example",
 *         keys: [
 *             "my_column_1",
 *             "my_column_2",
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Partition Indexes using the catalog ID (usually AWS account ID), database name, table name, and index name. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/partitionIndex:PartitionIndex example 123456789012:MyDatabase:MyTable:index-name
 * ```
 */
export declare class PartitionIndex extends pulumi.CustomResource {
    /**
     * Get an existing PartitionIndex resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PartitionIndexState, opts?: pulumi.CustomResourceOptions): PartitionIndex;
    /**
     * Returns true if the given object is an instance of PartitionIndex.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PartitionIndex;
    /**
     * The catalog ID where the table resides.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * Configuration block for a partition index. See `partitionIndex` below.
     */
    readonly partitionIndex: pulumi.Output<outputs.glue.PartitionIndexPartitionIndex>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the table. For Hive compatibility, this must be entirely lowercase.
     */
    readonly tableName: pulumi.Output<string>;
    /**
     * Create a PartitionIndex resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PartitionIndexArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PartitionIndex resources.
 */
export interface PartitionIndexState {
    /**
     * The catalog ID where the table resides.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * Configuration block for a partition index. See `partitionIndex` below.
     */
    partitionIndex?: pulumi.Input<inputs.glue.PartitionIndexPartitionIndex>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the table. For Hive compatibility, this must be entirely lowercase.
     */
    tableName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PartitionIndex resource.
 */
export interface PartitionIndexArgs {
    /**
     * The catalog ID where the table resides.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     */
    databaseName: pulumi.Input<string>;
    /**
     * Configuration block for a partition index. See `partitionIndex` below.
     */
    partitionIndex: pulumi.Input<inputs.glue.PartitionIndexPartitionIndex>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the table. For Hive compatibility, this must be entirely lowercase.
     */
    tableName: pulumi.Input<string>;
}

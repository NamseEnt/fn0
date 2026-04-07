import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue Catalog Table Resource. You can refer to the [Glue Developer Guide](http://docs.aws.amazon.com/glue/latest/dg/populate-data-catalog.html) for a full explanation of the Glue Data Catalog functionality.
 *
 * ## Example Usage
 *
 * ### Basic Table
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const awsGlueCatalogTable = new aws.glue.CatalogTable("aws_glue_catalog_table", {
 *     name: "MyCatalogTable",
 *     databaseName: "MyCatalogDatabase",
 * });
 * ```
 *
 * ### Parquet Table for Athena
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const awsGlueCatalogTable = new aws.glue.CatalogTable("aws_glue_catalog_table", {
 *     name: "MyCatalogTable",
 *     databaseName: "MyCatalogDatabase",
 *     tableType: "EXTERNAL_TABLE",
 *     parameters: {
 *         EXTERNAL: "TRUE",
 *         "parquet.compression": "SNAPPY",
 *     },
 *     storageDescriptor: {
 *         location: "s3://my-bucket/event-streams/my-stream",
 *         inputFormat: "org.apache.hadoop.hive.ql.io.parquet.MapredParquetInputFormat",
 *         outputFormat: "org.apache.hadoop.hive.ql.io.parquet.MapredParquetOutputFormat",
 *         serDeInfo: {
 *             name: "my-stream",
 *             serializationLibrary: "org.apache.hadoop.hive.ql.io.parquet.serde.ParquetHiveSerDe",
 *             parameters: {
 *                 "serialization.format": "1",
 *             },
 *         },
 *         columns: [
 *             {
 *                 name: "my_string",
 *                 type: "string",
 *             },
 *             {
 *                 name: "my_double",
 *                 type: "double",
 *             },
 *             {
 *                 name: "my_date",
 *                 type: "date",
 *                 comment: "",
 *             },
 *             {
 *                 name: "my_bigint",
 *                 type: "bigint",
 *                 comment: "",
 *             },
 *             {
 *                 name: "my_struct",
 *                 type: "struct<my_nested_string:string>",
 *                 comment: "",
 *             },
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Tables using the catalog ID (usually AWS account ID), database name, and table name. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/catalogTable:CatalogTable MyTable 123456789012:MyDatabase:MyTable
 * ```
 */
export declare class CatalogTable extends pulumi.CustomResource {
    /**
     * Get an existing CatalogTable resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CatalogTableState, opts?: pulumi.CustomResourceOptions): CatalogTable;
    /**
     * Returns true if the given object is an instance of CatalogTable.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CatalogTable;
    /**
     * The ARN of the Glue Table.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the Glue Catalog and database to create the table in. If omitted, this defaults to the AWS Account ID plus the database name.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     *
     * The following arguments are optional:
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * Description of the table.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the table. For Hive compatibility, this must be entirely lowercase.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Configuration block for open table formats. See `openTableFormatInput` below.
     */
    readonly openTableFormatInput: pulumi.Output<outputs.glue.CatalogTableOpenTableFormatInput | undefined>;
    /**
     * Owner of the table.
     */
    readonly owner: pulumi.Output<string | undefined>;
    /**
     * Properties associated with this table, as a list of key-value pairs.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Configuration block for a maximum of 3 partition indexes. See `partitionIndex` below.
     */
    readonly partitionIndices: pulumi.Output<outputs.glue.CatalogTablePartitionIndex[]>;
    /**
     * Configuration block of columns by which the table is partitioned. Only primitive types are supported as partition keys. See `partitionKeys` below.
     */
    readonly partitionKeys: pulumi.Output<outputs.glue.CatalogTablePartitionKey[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Retention time for this table.
     */
    readonly retention: pulumi.Output<number | undefined>;
    /**
     * Configuration block for information about the physical storage of this table. For more information, refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/aws-glue-api-catalog-tables.html#aws-glue-api-catalog-tables-StorageDescriptor). See `storageDescriptor` below.
     */
    readonly storageDescriptor: pulumi.Output<outputs.glue.CatalogTableStorageDescriptor | undefined>;
    /**
     * Type of this table (EXTERNAL_TABLE, VIRTUAL_VIEW, etc.). While optional, some Athena DDL queries such as `ALTER TABLE` and `SHOW CREATE TABLE` will fail if this argument is empty.
     */
    readonly tableType: pulumi.Output<string | undefined>;
    /**
     * Configuration block of a target table for resource linking. See `targetTable` below.
     */
    readonly targetTable: pulumi.Output<outputs.glue.CatalogTableTargetTable | undefined>;
    /**
     * If the table is a view, the expanded text of the view; otherwise null.
     */
    readonly viewExpandedText: pulumi.Output<string | undefined>;
    /**
     * If the table is a view, the original text of the view; otherwise null.
     */
    readonly viewOriginalText: pulumi.Output<string | undefined>;
    /**
     * Create a CatalogTable resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CatalogTableArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CatalogTable resources.
 */
export interface CatalogTableState {
    /**
     * The ARN of the Glue Table.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the Glue Catalog and database to create the table in. If omitted, this defaults to the AWS Account ID plus the database name.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     *
     * The following arguments are optional:
     */
    databaseName?: pulumi.Input<string>;
    /**
     * Description of the table.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the table. For Hive compatibility, this must be entirely lowercase.
     */
    name?: pulumi.Input<string>;
    /**
     * Configuration block for open table formats. See `openTableFormatInput` below.
     */
    openTableFormatInput?: pulumi.Input<inputs.glue.CatalogTableOpenTableFormatInput>;
    /**
     * Owner of the table.
     */
    owner?: pulumi.Input<string>;
    /**
     * Properties associated with this table, as a list of key-value pairs.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for a maximum of 3 partition indexes. See `partitionIndex` below.
     */
    partitionIndices?: pulumi.Input<pulumi.Input<inputs.glue.CatalogTablePartitionIndex>[]>;
    /**
     * Configuration block of columns by which the table is partitioned. Only primitive types are supported as partition keys. See `partitionKeys` below.
     */
    partitionKeys?: pulumi.Input<pulumi.Input<inputs.glue.CatalogTablePartitionKey>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Retention time for this table.
     */
    retention?: pulumi.Input<number>;
    /**
     * Configuration block for information about the physical storage of this table. For more information, refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/aws-glue-api-catalog-tables.html#aws-glue-api-catalog-tables-StorageDescriptor). See `storageDescriptor` below.
     */
    storageDescriptor?: pulumi.Input<inputs.glue.CatalogTableStorageDescriptor>;
    /**
     * Type of this table (EXTERNAL_TABLE, VIRTUAL_VIEW, etc.). While optional, some Athena DDL queries such as `ALTER TABLE` and `SHOW CREATE TABLE` will fail if this argument is empty.
     */
    tableType?: pulumi.Input<string>;
    /**
     * Configuration block of a target table for resource linking. See `targetTable` below.
     */
    targetTable?: pulumi.Input<inputs.glue.CatalogTableTargetTable>;
    /**
     * If the table is a view, the expanded text of the view; otherwise null.
     */
    viewExpandedText?: pulumi.Input<string>;
    /**
     * If the table is a view, the original text of the view; otherwise null.
     */
    viewOriginalText?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CatalogTable resource.
 */
export interface CatalogTableArgs {
    /**
     * ID of the Glue Catalog and database to create the table in. If omitted, this defaults to the AWS Account ID plus the database name.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     *
     * The following arguments are optional:
     */
    databaseName: pulumi.Input<string>;
    /**
     * Description of the table.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the table. For Hive compatibility, this must be entirely lowercase.
     */
    name?: pulumi.Input<string>;
    /**
     * Configuration block for open table formats. See `openTableFormatInput` below.
     */
    openTableFormatInput?: pulumi.Input<inputs.glue.CatalogTableOpenTableFormatInput>;
    /**
     * Owner of the table.
     */
    owner?: pulumi.Input<string>;
    /**
     * Properties associated with this table, as a list of key-value pairs.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for a maximum of 3 partition indexes. See `partitionIndex` below.
     */
    partitionIndices?: pulumi.Input<pulumi.Input<inputs.glue.CatalogTablePartitionIndex>[]>;
    /**
     * Configuration block of columns by which the table is partitioned. Only primitive types are supported as partition keys. See `partitionKeys` below.
     */
    partitionKeys?: pulumi.Input<pulumi.Input<inputs.glue.CatalogTablePartitionKey>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Retention time for this table.
     */
    retention?: pulumi.Input<number>;
    /**
     * Configuration block for information about the physical storage of this table. For more information, refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/aws-glue-api-catalog-tables.html#aws-glue-api-catalog-tables-StorageDescriptor). See `storageDescriptor` below.
     */
    storageDescriptor?: pulumi.Input<inputs.glue.CatalogTableStorageDescriptor>;
    /**
     * Type of this table (EXTERNAL_TABLE, VIRTUAL_VIEW, etc.). While optional, some Athena DDL queries such as `ALTER TABLE` and `SHOW CREATE TABLE` will fail if this argument is empty.
     */
    tableType?: pulumi.Input<string>;
    /**
     * Configuration block of a target table for resource linking. See `targetTable` below.
     */
    targetTable?: pulumi.Input<inputs.glue.CatalogTableTargetTable>;
    /**
     * If the table is a view, the expanded text of the view; otherwise null.
     */
    viewExpandedText?: pulumi.Input<string>;
    /**
     * If the table is a view, the original text of the view; otherwise null.
     */
    viewOriginalText?: pulumi.Input<string>;
}

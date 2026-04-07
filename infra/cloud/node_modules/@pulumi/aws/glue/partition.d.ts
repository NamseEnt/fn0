import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue Partition Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Partition("example", {
 *     databaseName: "some-database",
 *     tableName: "some-table",
 *     partitionValues: ["some-value"],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Partitions using the catalog ID (usually AWS account ID), database name, table name and partition values. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/partition:Partition part 123456789012:MyDatabase:MyTable:val1#val2
 * ```
 */
export declare class Partition extends pulumi.CustomResource {
    /**
     * Get an existing Partition resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PartitionState, opts?: pulumi.CustomResourceOptions): Partition;
    /**
     * Returns true if the given object is an instance of Partition.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Partition;
    /**
     * ID of the Glue Catalog and database to create the table in. If omitted, this defaults to the AWS Account ID plus the database name.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * The time at which the partition was created.
     */
    readonly creationTime: pulumi.Output<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * The last time at which the partition was accessed.
     */
    readonly lastAccessedTime: pulumi.Output<string>;
    /**
     * The last time at which column statistics were computed for this partition.
     */
    readonly lastAnalyzedTime: pulumi.Output<string>;
    /**
     * Properties associated with this table, as a list of key-value pairs.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The values that define the partition.
     */
    readonly partitionValues: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A storage descriptor object containing information about the physical storage of this table. You can refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/aws-glue-api-catalog-tables.html#aws-glue-api-catalog-tables-StorageDescriptor) for a full explanation of this object.
     */
    readonly storageDescriptor: pulumi.Output<outputs.glue.PartitionStorageDescriptor | undefined>;
    readonly tableName: pulumi.Output<string>;
    /**
     * Create a Partition resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PartitionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Partition resources.
 */
export interface PartitionState {
    /**
     * ID of the Glue Catalog and database to create the table in. If omitted, this defaults to the AWS Account ID plus the database name.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * The time at which the partition was created.
     */
    creationTime?: pulumi.Input<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * The last time at which the partition was accessed.
     */
    lastAccessedTime?: pulumi.Input<string>;
    /**
     * The last time at which column statistics were computed for this partition.
     */
    lastAnalyzedTime?: pulumi.Input<string>;
    /**
     * Properties associated with this table, as a list of key-value pairs.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The values that define the partition.
     */
    partitionValues?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A storage descriptor object containing information about the physical storage of this table. You can refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/aws-glue-api-catalog-tables.html#aws-glue-api-catalog-tables-StorageDescriptor) for a full explanation of this object.
     */
    storageDescriptor?: pulumi.Input<inputs.glue.PartitionStorageDescriptor>;
    tableName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Partition resource.
 */
export interface PartitionArgs {
    /**
     * ID of the Glue Catalog and database to create the table in. If omitted, this defaults to the AWS Account ID plus the database name.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Name of the metadata database where the table metadata resides. For Hive compatibility, this must be all lowercase.
     */
    databaseName: pulumi.Input<string>;
    /**
     * Properties associated with this table, as a list of key-value pairs.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The values that define the partition.
     */
    partitionValues: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A storage descriptor object containing information about the physical storage of this table. You can refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/aws-glue-api-catalog-tables.html#aws-glue-api-catalog-tables-StorageDescriptor) for a full explanation of this object.
     */
    storageDescriptor?: pulumi.Input<inputs.glue.PartitionStorageDescriptor>;
    tableName: pulumi.Input<string>;
}

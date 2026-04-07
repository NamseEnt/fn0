import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon S3 Tables Table.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleTableBucket = new aws.s3tables.TableBucket("example", {name: "example-bucket"});
 * const exampleNamespace = new aws.s3tables.Namespace("example", {
 *     namespace: "example_namespace",
 *     tableBucketArn: exampleTableBucket.arn,
 * });
 * const example = new aws.s3tables.Table("example", {
 *     name: "example_table",
 *     namespace: exampleNamespace.namespace,
 *     tableBucketArn: exampleNamespace.tableBucketArn,
 *     format: "ICEBERG",
 * });
 * ```
 *
 * ### With Metadata Schema
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleTableBucket = new aws.s3tables.TableBucket("example", {name: "example-bucket"});
 * const exampleNamespace = new aws.s3tables.Namespace("example", {
 *     namespace: "example_namespace",
 *     tableBucketArn: exampleTableBucket.arn,
 * });
 * const example = new aws.s3tables.Table("example", {
 *     name: "example_table",
 *     namespace: exampleNamespace.namespace,
 *     tableBucketArn: exampleNamespace.tableBucketArn,
 *     format: "ICEBERG",
 *     metadata: {
 *         iceberg: {
 *             schema: {
 *                 fields: [
 *                     {
 *                         name: "id",
 *                         type: "long",
 *                         required: true,
 *                     },
 *                     {
 *                         name: "name",
 *                         type: "string",
 *                         required: true,
 *                     },
 *                     {
 *                         name: "created_at",
 *                         type: "timestamp",
 *                         required: false,
 *                     },
 *                     {
 *                         name: "price",
 *                         type: "decimal(10,2)",
 *                         required: false,
 *                     },
 *                 ],
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Tables Table using the `table_bucket_arn`, the value of `namespace`, and the value of `name`, separated by a semicolon (`;`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3tables/table:Table example 'arn:aws:s3tables:us-west-2:123456789012:bucket/example-bucket;example-namespace;example-table'
 * ```
 */
export declare class Table extends pulumi.CustomResource {
    /**
     * Get an existing Table resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TableState, opts?: pulumi.CustomResourceOptions): Table;
    /**
     * Returns true if the given object is an instance of Table.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Table;
    /**
     * ARN of the table.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time when the namespace was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Account ID of the account that created the namespace.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * A single table bucket encryption configuration object.
     * See `encryptionConfiguration` below.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.s3tables.TableEncryptionConfiguration>;
    /**
     * Format of the table.
     * Must be `ICEBERG`.
     */
    readonly format: pulumi.Output<string>;
    /**
     * A single table bucket maintenance configuration object.
     * See `maintenanceConfiguration` below.
     */
    readonly maintenanceConfiguration: pulumi.Output<outputs.s3tables.TableMaintenanceConfiguration>;
    /**
     * Contains details about the table metadata. This configuration specifies the metadata format and schema for the table. Currently only supports Iceberg format.
     * See `metadata` below.
     */
    readonly metadata: pulumi.Output<outputs.s3tables.TableMetadata | undefined>;
    /**
     * Location of table metadata.
     */
    readonly metadataLocation: pulumi.Output<string>;
    /**
     * Date and time when the namespace was last modified.
     */
    readonly modifiedAt: pulumi.Output<string>;
    /**
     * Account ID of the account that last modified the namespace.
     */
    readonly modifiedBy: pulumi.Output<string>;
    /**
     * Name of the table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     * A full list of table naming rules can be found in the [S3 Tables documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-tables-buckets-naming.html#naming-rules-table).
     */
    readonly name: pulumi.Output<string>;
    /**
     * Name of the namespace for this table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * Account ID of the account that owns the namespace.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     *
     * The following arguments are optional:
     */
    readonly tableBucketArn: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Type of the table.
     * One of `customer` or `aws`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Identifier for the current version of table data.
     */
    readonly versionToken: pulumi.Output<string>;
    /**
     * S3 URI pointing to the S3 Bucket that contains the table data.
     */
    readonly warehouseLocation: pulumi.Output<string>;
    /**
     * Create a Table resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TableArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Table resources.
 */
export interface TableState {
    /**
     * ARN of the table.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time when the namespace was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Account ID of the account that created the namespace.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * A single table bucket encryption configuration object.
     * See `encryptionConfiguration` below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.s3tables.TableEncryptionConfiguration>;
    /**
     * Format of the table.
     * Must be `ICEBERG`.
     */
    format?: pulumi.Input<string>;
    /**
     * A single table bucket maintenance configuration object.
     * See `maintenanceConfiguration` below.
     */
    maintenanceConfiguration?: pulumi.Input<inputs.s3tables.TableMaintenanceConfiguration>;
    /**
     * Contains details about the table metadata. This configuration specifies the metadata format and schema for the table. Currently only supports Iceberg format.
     * See `metadata` below.
     */
    metadata?: pulumi.Input<inputs.s3tables.TableMetadata>;
    /**
     * Location of table metadata.
     */
    metadataLocation?: pulumi.Input<string>;
    /**
     * Date and time when the namespace was last modified.
     */
    modifiedAt?: pulumi.Input<string>;
    /**
     * Account ID of the account that last modified the namespace.
     */
    modifiedBy?: pulumi.Input<string>;
    /**
     * Name of the table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     * A full list of table naming rules can be found in the [S3 Tables documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-tables-buckets-naming.html#naming-rules-table).
     */
    name?: pulumi.Input<string>;
    /**
     * Name of the namespace for this table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Account ID of the account that owns the namespace.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     *
     * The following arguments are optional:
     */
    tableBucketArn?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of the table.
     * One of `customer` or `aws`.
     */
    type?: pulumi.Input<string>;
    /**
     * Identifier for the current version of table data.
     */
    versionToken?: pulumi.Input<string>;
    /**
     * S3 URI pointing to the S3 Bucket that contains the table data.
     */
    warehouseLocation?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Table resource.
 */
export interface TableArgs {
    /**
     * A single table bucket encryption configuration object.
     * See `encryptionConfiguration` below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.s3tables.TableEncryptionConfiguration>;
    /**
     * Format of the table.
     * Must be `ICEBERG`.
     */
    format: pulumi.Input<string>;
    /**
     * A single table bucket maintenance configuration object.
     * See `maintenanceConfiguration` below.
     */
    maintenanceConfiguration?: pulumi.Input<inputs.s3tables.TableMaintenanceConfiguration>;
    /**
     * Contains details about the table metadata. This configuration specifies the metadata format and schema for the table. Currently only supports Iceberg format.
     * See `metadata` below.
     */
    metadata?: pulumi.Input<inputs.s3tables.TableMetadata>;
    /**
     * Name of the table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     * A full list of table naming rules can be found in the [S3 Tables documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-tables-buckets-naming.html#naming-rules-table).
     */
    name?: pulumi.Input<string>;
    /**
     * Name of the namespace for this table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    namespace: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     *
     * The following arguments are optional:
     */
    tableBucketArn: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

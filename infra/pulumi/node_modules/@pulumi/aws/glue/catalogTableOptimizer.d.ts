import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Glue Catalog Table Optimizer.
 *
 * ## Example Usage
 *
 * ### Compaction Optimizer
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogTableOptimizer("example", {
 *     catalogId: "123456789012",
 *     databaseName: "example_database",
 *     tableName: "example_table",
 *     configuration: {
 *         roleArn: "arn:aws:iam::123456789012:role/example-role",
 *         enabled: true,
 *     },
 *     type: "compaction",
 * });
 * ```
 *
 * ### Snapshot Retention Optimizer
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogTableOptimizer("example", {
 *     catalogId: "123456789012",
 *     databaseName: "example_database",
 *     tableName: "example_table",
 *     configuration: {
 *         roleArn: "arn:aws:iam::123456789012:role/example-role",
 *         enabled: true,
 *         retentionConfiguration: {
 *             icebergConfiguration: {
 *                 snapshotRetentionPeriodInDays: 7,
 *                 numberOfSnapshotsToRetain: 3,
 *                 cleanExpiredFiles: true,
 *             },
 *         },
 *     },
 *     type: "retention",
 * });
 * ```
 *
 * ### Orphan File Deletion Optimizer
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogTableOptimizer("example", {
 *     catalogId: "123456789012",
 *     databaseName: "example_database",
 *     tableName: "example_table",
 *     configuration: {
 *         roleArn: "arn:aws:iam::123456789012:role/example-role",
 *         enabled: true,
 *         orphanFileDeletionConfiguration: {
 *             icebergConfiguration: {
 *                 orphanFileRetentionPeriodInDays: 7,
 *                 location: "s3://example-bucket/example_table/",
 *             },
 *         },
 *     },
 *     type: "orphan_file_deletion",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Catalog Table Optimizer using the `catalog_id,database_name,table_name,type`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/catalogTableOptimizer:CatalogTableOptimizer example 123456789012,example_database,example_table,compaction
 * ```
 */
export declare class CatalogTableOptimizer extends pulumi.CustomResource {
    /**
     * Get an existing CatalogTableOptimizer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CatalogTableOptimizerState, opts?: pulumi.CustomResourceOptions): CatalogTableOptimizer;
    /**
     * Returns true if the given object is an instance of CatalogTableOptimizer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CatalogTableOptimizer;
    /**
     * The Catalog ID of the table.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * A configuration block that defines the table optimizer settings. See Configuration for additional details.
     */
    readonly configuration: pulumi.Output<outputs.glue.CatalogTableOptimizerConfiguration | undefined>;
    /**
     * The name of the database in the catalog in which the table resides.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the table.
     */
    readonly tableName: pulumi.Output<string>;
    /**
     * The type of table optimizer. Valid values are `compaction`, `retention`, and `orphanFileDeletion`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a CatalogTableOptimizer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CatalogTableOptimizerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CatalogTableOptimizer resources.
 */
export interface CatalogTableOptimizerState {
    /**
     * The Catalog ID of the table.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * A configuration block that defines the table optimizer settings. See Configuration for additional details.
     */
    configuration?: pulumi.Input<inputs.glue.CatalogTableOptimizerConfiguration>;
    /**
     * The name of the database in the catalog in which the table resides.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the table.
     */
    tableName?: pulumi.Input<string>;
    /**
     * The type of table optimizer. Valid values are `compaction`, `retention`, and `orphanFileDeletion`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CatalogTableOptimizer resource.
 */
export interface CatalogTableOptimizerArgs {
    /**
     * The Catalog ID of the table.
     */
    catalogId: pulumi.Input<string>;
    /**
     * A configuration block that defines the table optimizer settings. See Configuration for additional details.
     */
    configuration?: pulumi.Input<inputs.glue.CatalogTableOptimizerConfiguration>;
    /**
     * The name of the database in the catalog in which the table resides.
     */
    databaseName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the table.
     */
    tableName: pulumi.Input<string>;
    /**
     * The type of table optimizer. Valid values are `compaction`, `retention`, and `orphanFileDeletion`.
     */
    type: pulumi.Input<string>;
}

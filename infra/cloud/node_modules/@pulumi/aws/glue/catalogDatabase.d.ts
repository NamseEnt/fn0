import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue Catalog Database Resource. You can refer to the [Glue Developer Guide](http://docs.aws.amazon.com/glue/latest/dg/populate-data-catalog.html) for a full explanation of the Glue Data Catalog functionality
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogDatabase("example", {name: "MyCatalogDatabase"});
 * ```
 *
 * ### Create Table Default Permissions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogDatabase("example", {
 *     name: "MyCatalogDatabase",
 *     createTableDefaultPermissions: [{
 *         permissions: ["SELECT"],
 *         principal: {
 *             dataLakePrincipalIdentifier: "IAM_ALLOWED_PRINCIPALS",
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Catalog Databases using the `catalog_id:name`. If you have not set a Catalog ID specify the AWS Account ID that the database is in. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/catalogDatabase:CatalogDatabase database 123456789012:my_database
 * ```
 */
export declare class CatalogDatabase extends pulumi.CustomResource {
    /**
     * Get an existing CatalogDatabase resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CatalogDatabaseState, opts?: pulumi.CustomResourceOptions): CatalogDatabase;
    /**
     * Returns true if the given object is an instance of CatalogDatabase.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CatalogDatabase;
    /**
     * ARN of the Glue Catalog Database.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the Glue Catalog to create the database in. If omitted, this defaults to the AWS Account ID.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * Creates a set of default permissions on the table for principals. See `createTableDefaultPermission` below.
     */
    readonly createTableDefaultPermissions: pulumi.Output<outputs.glue.CatalogDatabaseCreateTableDefaultPermission[]>;
    /**
     * Description of the database.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Configuration block that references an entity outside the AWS Glue Data Catalog. See `federatedDatabase` below.
     */
    readonly federatedDatabase: pulumi.Output<outputs.glue.CatalogDatabaseFederatedDatabase | undefined>;
    /**
     * Location of the database (for example, an HDFS path).
     */
    readonly locationUri: pulumi.Output<string>;
    /**
     * Name of the database. The acceptable characters are lowercase letters, numbers, and the underscore character.
     */
    readonly name: pulumi.Output<string>;
    /**
     * List of key-value pairs that define parameters and properties of the database.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Configuration block for a target database for resource linking. See `targetDatabase` below.
     */
    readonly targetDatabase: pulumi.Output<outputs.glue.CatalogDatabaseTargetDatabase | undefined>;
    /**
     * Create a CatalogDatabase resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: CatalogDatabaseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CatalogDatabase resources.
 */
export interface CatalogDatabaseState {
    /**
     * ARN of the Glue Catalog Database.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the Glue Catalog to create the database in. If omitted, this defaults to the AWS Account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Creates a set of default permissions on the table for principals. See `createTableDefaultPermission` below.
     */
    createTableDefaultPermissions?: pulumi.Input<pulumi.Input<inputs.glue.CatalogDatabaseCreateTableDefaultPermission>[]>;
    /**
     * Description of the database.
     */
    description?: pulumi.Input<string>;
    /**
     * Configuration block that references an entity outside the AWS Glue Data Catalog. See `federatedDatabase` below.
     */
    federatedDatabase?: pulumi.Input<inputs.glue.CatalogDatabaseFederatedDatabase>;
    /**
     * Location of the database (for example, an HDFS path).
     */
    locationUri?: pulumi.Input<string>;
    /**
     * Name of the database. The acceptable characters are lowercase letters, numbers, and the underscore character.
     */
    name?: pulumi.Input<string>;
    /**
     * List of key-value pairs that define parameters and properties of the database.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
     * Configuration block for a target database for resource linking. See `targetDatabase` below.
     */
    targetDatabase?: pulumi.Input<inputs.glue.CatalogDatabaseTargetDatabase>;
}
/**
 * The set of arguments for constructing a CatalogDatabase resource.
 */
export interface CatalogDatabaseArgs {
    /**
     * ID of the Glue Catalog to create the database in. If omitted, this defaults to the AWS Account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Creates a set of default permissions on the table for principals. See `createTableDefaultPermission` below.
     */
    createTableDefaultPermissions?: pulumi.Input<pulumi.Input<inputs.glue.CatalogDatabaseCreateTableDefaultPermission>[]>;
    /**
     * Description of the database.
     */
    description?: pulumi.Input<string>;
    /**
     * Configuration block that references an entity outside the AWS Glue Data Catalog. See `federatedDatabase` below.
     */
    federatedDatabase?: pulumi.Input<inputs.glue.CatalogDatabaseFederatedDatabase>;
    /**
     * Location of the database (for example, an HDFS path).
     */
    locationUri?: pulumi.Input<string>;
    /**
     * Name of the database. The acceptable characters are lowercase letters, numbers, and the underscore character.
     */
    name?: pulumi.Input<string>;
    /**
     * List of key-value pairs that define parameters and properties of the database.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for a target database for resource linking. See `targetDatabase` below.
     */
    targetDatabase?: pulumi.Input<inputs.glue.CatalogDatabaseTargetDatabase>;
}

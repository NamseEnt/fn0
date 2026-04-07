import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail database. Use this resource to create and manage fully managed database instances with automated backups, monitoring, and maintenance in Lightsail.
 *
 * > **Note:** Lightsail is currently only supported in a limited number of AWS Regions, please see ["Regions and Availability Zones"](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services/) for more details
 *
 * ## Example Usage
 *
 * ### Basic MySQL Blueprint
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Database("example", {
 *     relationalDatabaseName: "example-database",
 *     availabilityZone: "us-east-1a",
 *     masterDatabaseName: "exampledb",
 *     masterPassword: "examplepassword123",
 *     masterUsername: "exampleuser",
 *     blueprintId: "mysql_8_0",
 *     bundleId: "micro_1_0",
 * });
 * ```
 *
 * ### Basic PostgreSQL Blueprint
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Database("example", {
 *     relationalDatabaseName: "example-database",
 *     availabilityZone: "us-east-1a",
 *     masterDatabaseName: "exampledb",
 *     masterPassword: "examplepassword123",
 *     masterUsername: "exampleuser",
 *     blueprintId: "postgres_12",
 *     bundleId: "micro_1_0",
 * });
 * ```
 *
 * ### Custom Backup and Maintenance Windows
 *
 * Below is an example that sets a custom backup and maintenance window. Times are specified in UTC. This example will allow daily backups to take place between 16:00 and 16:30 each day. This example also requires any maintenance tasks (anything that would cause an outage, including changing some attributes) to take place on Tuesdays between 17:00 and 17:30. An action taken against this database that would cause an outage will wait until this time window to make the requested changes.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Database("example", {
 *     relationalDatabaseName: "example-database",
 *     availabilityZone: "us-east-1a",
 *     masterDatabaseName: "exampledb",
 *     masterPassword: "examplepassword123",
 *     masterUsername: "exampleuser",
 *     blueprintId: "postgres_12",
 *     bundleId: "micro_1_0",
 *     preferredBackupWindow: "16:00-16:30",
 *     preferredMaintenanceWindow: "Tue:17:00-Tue:17:30",
 * });
 * ```
 *
 * ### Final Snapshots
 *
 * To enable creating a final snapshot of your database on deletion, use the `finalSnapshotName` argument to provide a name to be used for the snapshot.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Database("example", {
 *     relationalDatabaseName: "example-database",
 *     availabilityZone: "us-east-1a",
 *     masterDatabaseName: "exampledb",
 *     masterPassword: "examplepassword123",
 *     masterUsername: "exampleuser",
 *     blueprintId: "postgres_12",
 *     bundleId: "micro_1_0",
 *     preferredBackupWindow: "16:00-16:30",
 *     preferredMaintenanceWindow: "Tue:17:00-Tue:17:30",
 *     finalSnapshotName: "example-final-snapshot",
 * });
 * ```
 *
 * ### Apply Immediately
 *
 * To enable applying changes immediately instead of waiting for a maintenance window, use the `applyImmediately` argument.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Database("example", {
 *     relationalDatabaseName: "example-database",
 *     availabilityZone: "us-east-1a",
 *     masterDatabaseName: "exampledb",
 *     masterPassword: "examplepassword123",
 *     masterUsername: "exampleuser",
 *     blueprintId: "postgres_12",
 *     bundleId: "micro_1_0",
 *     applyImmediately: true,
 * });
 * ```
 *
 * ## Blueprint IDs
 *
 * A list of all available Lightsail Blueprints for Relational Databases the [aws lightsail get-relational-database-blueprints](https://docs.aws.amazon.com/cli/latest/reference/lightsail/get-relational-database-blueprints.html) aws cli command.
 *
 * ### Examples
 *
 * - `mysql80`
 * - `postgres12`
 *
 * ### Prefix
 *
 * A Blueprint ID starts with a prefix of the engine type.
 *
 * ### Suffix
 *
 * A Blueprint ID has a suffix of the engine version.
 *
 * ## Bundles
 *
 * A list of all available Lightsail Bundles for Relational Databases the [aws lightsail get-relational-database-bundles](https://docs.aws.amazon.com/cli/latest/reference/lightsail/get-relational-database-bundles.html) aws cli command.
 *
 * ### Examples
 *
 * - `small10`
 * - `smallHa10`
 * - `large10`
 * - `largeHa10`
 *
 * ### Prefix
 *
 * A Bundle ID starts with one of the below size prefixes:
 *
 * - `micro_`
 * - `small_`
 * - `medium_`
 * - `large_`
 *
 * ### Infixes (Optional for HA Database)
 *
 * A Bundle ID can have the following infix added in order to use the HA option of the selected bundle.
 *
 * - `ha_`
 *
 * ### Suffix
 *
 * A Bundle ID ends with one of the following suffix: `10`
 *
 * ## Import
 *
 * Using `pulumi import`, import Lightsail Databases using their name. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/database:Database example example-database
 * ```
 */
export declare class Database extends pulumi.CustomResource {
    /**
     * Get an existing Database resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DatabaseState, opts?: pulumi.CustomResourceOptions): Database;
    /**
     * Returns true if the given object is an instance of Database.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Database;
    /**
     * Whether to apply changes immediately. When false, applies changes during the preferred maintenance window. Some changes may cause an outage.
     */
    readonly applyImmediately: pulumi.Output<boolean>;
    /**
     * ARN of the database (matches `id`).
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Availability Zone in which to create your database. Use the us-east-2a case-sensitive format.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Whether to enable automated backup retention for your database. When false, disables automated backup retention for your database. Disabling backup retention deletes all automated database backups. Before disabling this, you may want to create a snapshot of your database.
     */
    readonly backupRetentionEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Blueprint ID for your database. A blueprint describes the major engine version of a database. You can get a list of database blueprints IDs by using the AWS CLI command: `aws lightsail get-relational-database-blueprints`
     */
    readonly blueprintId: pulumi.Output<string>;
    /**
     * Bundle ID for your database. A bundle describes the performance specifications for your database (see list below). You can get a list of database bundle IDs by using the AWS CLI command: `aws lightsail get-relational-database-bundles`.
     */
    readonly bundleId: pulumi.Output<string>;
    /**
     * Certificate associated with the database.
     */
    readonly caCertificateIdentifier: pulumi.Output<string>;
    /**
     * Number of vCPUs for the database.
     */
    readonly cpuCount: pulumi.Output<number>;
    /**
     * Date and time when the database was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Size of the disk for the database.
     */
    readonly diskSize: pulumi.Output<number>;
    /**
     * Database software (for example, MySQL).
     */
    readonly engine: pulumi.Output<string>;
    /**
     * Database engine version (for example, 5.7.23).
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Name of the database snapshot created if skip final snapshot is false, which is the default value for that parameter.
     */
    readonly finalSnapshotName: pulumi.Output<string | undefined>;
    /**
     * Name of the master database created when the Lightsail database resource is created.
     */
    readonly masterDatabaseName: pulumi.Output<string>;
    /**
     * Master endpoint FQDN for the database.
     */
    readonly masterEndpointAddress: pulumi.Output<string>;
    /**
     * Master endpoint network port for the database.
     */
    readonly masterEndpointPort: pulumi.Output<number>;
    /**
     * Password for the master user of your database. The password can include any printable ASCII character except "/", """, or "@".
     */
    readonly masterPassword: pulumi.Output<string>;
    /**
     * Master user name for your database.
     */
    readonly masterUsername: pulumi.Output<string>;
    /**
     * Daily time range during which automated backups are created for your database if automated backups are enabled. Must be in the hh24:mi-hh24:mi format. Example: `16:00-16:30`. Specified in Coordinated Universal Time (UTC).
     */
    readonly preferredBackupWindow: pulumi.Output<string>;
    /**
     * Weekly time range during which system maintenance can occur on your database. Must be in the ddd:hh24:mi-ddd:hh24:mi format. Specified in Coordinated Universal Time (UTC). Example: `Tue:17:00-Tue:17:30`
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * Whether the database is accessible to resources outside of your Lightsail account. A value of true specifies a database that is available to resources outside of your Lightsail account. A value of false specifies a database that is available only to your Lightsail resources in the same region as your database.
     */
    readonly publiclyAccessible: pulumi.Output<boolean | undefined>;
    /**
     * Amount of RAM in GB for the database.
     */
    readonly ramSize: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name to use for your Lightsail database resource. Names be unique within each AWS Region in your Lightsail account.
     *
     * The following arguments are optional:
     */
    readonly relationalDatabaseName: pulumi.Output<string>;
    /**
     * Secondary Availability Zone of a high availability database. The secondary database is used for failover support of a high availability database.
     */
    readonly secondaryAvailabilityZone: pulumi.Output<string>;
    /**
     * Whether a final database snapshot is created before your database is deleted. If true is specified, no database snapshot is created. If false is specified, a database snapshot is created before your database is deleted. You must specify the final relational database snapshot name parameter if the skip final snapshot parameter is false.
     */
    readonly skipFinalSnapshot: pulumi.Output<boolean | undefined>;
    /**
     * Support code for the database. Include this code in your email to support when you have questions about a database in Lightsail. This code enables our support team to look up your Lightsail information more easily.
     */
    readonly supportCode: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Database resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DatabaseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Database resources.
 */
export interface DatabaseState {
    /**
     * Whether to apply changes immediately. When false, applies changes during the preferred maintenance window. Some changes may cause an outage.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * ARN of the database (matches `id`).
     */
    arn?: pulumi.Input<string>;
    /**
     * Availability Zone in which to create your database. Use the us-east-2a case-sensitive format.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Whether to enable automated backup retention for your database. When false, disables automated backup retention for your database. Disabling backup retention deletes all automated database backups. Before disabling this, you may want to create a snapshot of your database.
     */
    backupRetentionEnabled?: pulumi.Input<boolean>;
    /**
     * Blueprint ID for your database. A blueprint describes the major engine version of a database. You can get a list of database blueprints IDs by using the AWS CLI command: `aws lightsail get-relational-database-blueprints`
     */
    blueprintId?: pulumi.Input<string>;
    /**
     * Bundle ID for your database. A bundle describes the performance specifications for your database (see list below). You can get a list of database bundle IDs by using the AWS CLI command: `aws lightsail get-relational-database-bundles`.
     */
    bundleId?: pulumi.Input<string>;
    /**
     * Certificate associated with the database.
     */
    caCertificateIdentifier?: pulumi.Input<string>;
    /**
     * Number of vCPUs for the database.
     */
    cpuCount?: pulumi.Input<number>;
    /**
     * Date and time when the database was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Size of the disk for the database.
     */
    diskSize?: pulumi.Input<number>;
    /**
     * Database software (for example, MySQL).
     */
    engine?: pulumi.Input<string>;
    /**
     * Database engine version (for example, 5.7.23).
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Name of the database snapshot created if skip final snapshot is false, which is the default value for that parameter.
     */
    finalSnapshotName?: pulumi.Input<string>;
    /**
     * Name of the master database created when the Lightsail database resource is created.
     */
    masterDatabaseName?: pulumi.Input<string>;
    /**
     * Master endpoint FQDN for the database.
     */
    masterEndpointAddress?: pulumi.Input<string>;
    /**
     * Master endpoint network port for the database.
     */
    masterEndpointPort?: pulumi.Input<number>;
    /**
     * Password for the master user of your database. The password can include any printable ASCII character except "/", """, or "@".
     */
    masterPassword?: pulumi.Input<string>;
    /**
     * Master user name for your database.
     */
    masterUsername?: pulumi.Input<string>;
    /**
     * Daily time range during which automated backups are created for your database if automated backups are enabled. Must be in the hh24:mi-hh24:mi format. Example: `16:00-16:30`. Specified in Coordinated Universal Time (UTC).
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Weekly time range during which system maintenance can occur on your database. Must be in the ddd:hh24:mi-ddd:hh24:mi format. Specified in Coordinated Universal Time (UTC). Example: `Tue:17:00-Tue:17:30`
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Whether the database is accessible to resources outside of your Lightsail account. A value of true specifies a database that is available to resources outside of your Lightsail account. A value of false specifies a database that is available only to your Lightsail resources in the same region as your database.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Amount of RAM in GB for the database.
     */
    ramSize?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name to use for your Lightsail database resource. Names be unique within each AWS Region in your Lightsail account.
     *
     * The following arguments are optional:
     */
    relationalDatabaseName?: pulumi.Input<string>;
    /**
     * Secondary Availability Zone of a high availability database. The secondary database is used for failover support of a high availability database.
     */
    secondaryAvailabilityZone?: pulumi.Input<string>;
    /**
     * Whether a final database snapshot is created before your database is deleted. If true is specified, no database snapshot is created. If false is specified, a database snapshot is created before your database is deleted. You must specify the final relational database snapshot name parameter if the skip final snapshot parameter is false.
     */
    skipFinalSnapshot?: pulumi.Input<boolean>;
    /**
     * Support code for the database. Include this code in your email to support when you have questions about a database in Lightsail. This code enables our support team to look up your Lightsail information more easily.
     */
    supportCode?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Database resource.
 */
export interface DatabaseArgs {
    /**
     * Whether to apply changes immediately. When false, applies changes during the preferred maintenance window. Some changes may cause an outage.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * Availability Zone in which to create your database. Use the us-east-2a case-sensitive format.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Whether to enable automated backup retention for your database. When false, disables automated backup retention for your database. Disabling backup retention deletes all automated database backups. Before disabling this, you may want to create a snapshot of your database.
     */
    backupRetentionEnabled?: pulumi.Input<boolean>;
    /**
     * Blueprint ID for your database. A blueprint describes the major engine version of a database. You can get a list of database blueprints IDs by using the AWS CLI command: `aws lightsail get-relational-database-blueprints`
     */
    blueprintId: pulumi.Input<string>;
    /**
     * Bundle ID for your database. A bundle describes the performance specifications for your database (see list below). You can get a list of database bundle IDs by using the AWS CLI command: `aws lightsail get-relational-database-bundles`.
     */
    bundleId: pulumi.Input<string>;
    /**
     * Name of the database snapshot created if skip final snapshot is false, which is the default value for that parameter.
     */
    finalSnapshotName?: pulumi.Input<string>;
    /**
     * Name of the master database created when the Lightsail database resource is created.
     */
    masterDatabaseName: pulumi.Input<string>;
    /**
     * Password for the master user of your database. The password can include any printable ASCII character except "/", """, or "@".
     */
    masterPassword: pulumi.Input<string>;
    /**
     * Master user name for your database.
     */
    masterUsername: pulumi.Input<string>;
    /**
     * Daily time range during which automated backups are created for your database if automated backups are enabled. Must be in the hh24:mi-hh24:mi format. Example: `16:00-16:30`. Specified in Coordinated Universal Time (UTC).
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Weekly time range during which system maintenance can occur on your database. Must be in the ddd:hh24:mi-ddd:hh24:mi format. Specified in Coordinated Universal Time (UTC). Example: `Tue:17:00-Tue:17:30`
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Whether the database is accessible to resources outside of your Lightsail account. A value of true specifies a database that is available to resources outside of your Lightsail account. A value of false specifies a database that is available only to your Lightsail resources in the same region as your database.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name to use for your Lightsail database resource. Names be unique within each AWS Region in your Lightsail account.
     *
     * The following arguments are optional:
     */
    relationalDatabaseName: pulumi.Input<string>;
    /**
     * Whether a final database snapshot is created before your database is deleted. If true is specified, no database snapshot is created. If false is specified, a database snapshot is created before your database is deleted. You must specify the final relational database snapshot name parameter if the skip final snapshot parameter is false.
     */
    skipFinalSnapshot?: pulumi.Input<boolean>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS FinSpace Kx Dataview.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.finspace.KxDataview("example", {
 *     name: "my-tf-kx-dataview",
 *     environmentId: exampleAwsFinspaceKxEnvironment.id,
 *     databaseName: exampleAwsFinspaceKxDatabase.name,
 *     availabilityZoneId: "use1-az2",
 *     description: "Terraform managed Kx Dataview",
 *     azMode: "SINGLE",
 *     autoUpdate: true,
 *     segmentConfigurations: [{
 *         volumeName: exampleAwsFinspaceKxVolume.name,
 *         dbPaths: ["/*"],
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an AWS FinSpace Kx Cluster using the `id` (environment ID and cluster name, comma-delimited). For example:
 *
 * ```sh
 * $ pulumi import aws:finspace/kxDataview:KxDataview example n3ceo7wqxoxcti5tujqwzs,my-tf-kx-database,my-tf-kx-dataview
 * ```
 */
export declare class KxDataview extends pulumi.CustomResource {
    /**
     * Get an existing KxDataview resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KxDataviewState, opts?: pulumi.CustomResourceOptions): KxDataview;
    /**
     * Returns true if the given object is an instance of KxDataview.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KxDataview;
    /**
     * Amazon Resource Name (ARN) identifier of the KX dataview.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The option to specify whether you want to apply all the future additions and corrections automatically to the dataview, when you ingest new changesets. The default value is false.
     */
    readonly autoUpdate: pulumi.Output<boolean>;
    /**
     * The identifier of the availability zones. If attaching a volume, the volume must be in the same availability zone as the dataview that you are attaching to.
     */
    readonly availabilityZoneId: pulumi.Output<string | undefined>;
    /**
     * The number of availability zones you want to assign per cluster. This can be one of the following:
     * * `SINGLE` - Assigns one availability zone per cluster.
     * * `MULTI` - Assigns all the availability zones per cluster.
     */
    readonly azMode: pulumi.Output<string>;
    /**
     * A unique identifier of the changeset of the database that you want to use to ingest data.
     */
    readonly changesetId: pulumi.Output<string | undefined>;
    /**
     * Timestamp at which the dataview was created in FinSpace. Value determined as epoch time in milliseconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000000.
     */
    readonly createdTimestamp: pulumi.Output<string>;
    /**
     * The name of the database where you want to create a dataview.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * A description for the dataview.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Unique identifier for the KX environment.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * The last time that the dataview was updated in FinSpace. The value is determined as epoch time in milliseconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000000.
     */
    readonly lastModifiedTimestamp: pulumi.Output<string>;
    /**
     * A unique identifier for the dataview.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * The option to specify whether you want to make the dataview writable to perform database maintenance. The following are some considerations related to writable dataviews.
     * * You cannot create partial writable dataviews. When you create writeable dataviews you must provide the entire database path. You cannot perform updates on a writeable dataview. Hence, `autoUpdate` must be set as `false` if `readWrite` is `true` for a dataview.
     * * You must also use a unique volume for creating a writeable dataview. So, if you choose a volume that is already in use by another dataview, the dataview creation fails.
     * * Once you create a dataview as writeable, you cannot change it to read-only. So, you cannot update the `readWrite` parameter later.
     */
    readonly readWrite: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The configuration that contains the database path of the data that you want to place on each selected volume. Each segment must have a unique database path for each volume. If you do not explicitly specify any database path for a volume, they are accessible from the cluster through the default S3/object store segment. See segmentConfigurations below.
     */
    readonly segmentConfigurations: pulumi.Output<outputs.finspace.KxDataviewSegmentConfiguration[] | undefined>;
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a KxDataview resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KxDataviewArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KxDataview resources.
 */
export interface KxDataviewState {
    /**
     * Amazon Resource Name (ARN) identifier of the KX dataview.
     */
    arn?: pulumi.Input<string>;
    /**
     * The option to specify whether you want to apply all the future additions and corrections automatically to the dataview, when you ingest new changesets. The default value is false.
     */
    autoUpdate?: pulumi.Input<boolean>;
    /**
     * The identifier of the availability zones. If attaching a volume, the volume must be in the same availability zone as the dataview that you are attaching to.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The number of availability zones you want to assign per cluster. This can be one of the following:
     * * `SINGLE` - Assigns one availability zone per cluster.
     * * `MULTI` - Assigns all the availability zones per cluster.
     */
    azMode?: pulumi.Input<string>;
    /**
     * A unique identifier of the changeset of the database that you want to use to ingest data.
     */
    changesetId?: pulumi.Input<string>;
    /**
     * Timestamp at which the dataview was created in FinSpace. Value determined as epoch time in milliseconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000000.
     */
    createdTimestamp?: pulumi.Input<string>;
    /**
     * The name of the database where you want to create a dataview.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * A description for the dataview.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * The last time that the dataview was updated in FinSpace. The value is determined as epoch time in milliseconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000000.
     */
    lastModifiedTimestamp?: pulumi.Input<string>;
    /**
     * A unique identifier for the dataview.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The option to specify whether you want to make the dataview writable to perform database maintenance. The following are some considerations related to writable dataviews.
     * * You cannot create partial writable dataviews. When you create writeable dataviews you must provide the entire database path. You cannot perform updates on a writeable dataview. Hence, `autoUpdate` must be set as `false` if `readWrite` is `true` for a dataview.
     * * You must also use a unique volume for creating a writeable dataview. So, if you choose a volume that is already in use by another dataview, the dataview creation fails.
     * * Once you create a dataview as writeable, you cannot change it to read-only. So, you cannot update the `readWrite` parameter later.
     */
    readWrite?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration that contains the database path of the data that you want to place on each selected volume. Each segment must have a unique database path for each volume. If you do not explicitly specify any database path for a volume, they are accessible from the cluster through the default S3/object store segment. See segmentConfigurations below.
     */
    segmentConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxDataviewSegmentConfiguration>[]>;
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a KxDataview resource.
 */
export interface KxDataviewArgs {
    /**
     * The option to specify whether you want to apply all the future additions and corrections automatically to the dataview, when you ingest new changesets. The default value is false.
     */
    autoUpdate: pulumi.Input<boolean>;
    /**
     * The identifier of the availability zones. If attaching a volume, the volume must be in the same availability zone as the dataview that you are attaching to.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The number of availability zones you want to assign per cluster. This can be one of the following:
     * * `SINGLE` - Assigns one availability zone per cluster.
     * * `MULTI` - Assigns all the availability zones per cluster.
     */
    azMode: pulumi.Input<string>;
    /**
     * A unique identifier of the changeset of the database that you want to use to ingest data.
     */
    changesetId?: pulumi.Input<string>;
    /**
     * The name of the database where you want to create a dataview.
     */
    databaseName: pulumi.Input<string>;
    /**
     * A description for the dataview.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId: pulumi.Input<string>;
    /**
     * A unique identifier for the dataview.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The option to specify whether you want to make the dataview writable to perform database maintenance. The following are some considerations related to writable dataviews.
     * * You cannot create partial writable dataviews. When you create writeable dataviews you must provide the entire database path. You cannot perform updates on a writeable dataview. Hence, `autoUpdate` must be set as `false` if `readWrite` is `true` for a dataview.
     * * You must also use a unique volume for creating a writeable dataview. So, if you choose a volume that is already in use by another dataview, the dataview creation fails.
     * * Once you create a dataview as writeable, you cannot change it to read-only. So, you cannot update the `readWrite` parameter later.
     */
    readWrite?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration that contains the database path of the data that you want to place on each selected volume. Each segment must have a unique database path for each volume. If you do not explicitly specify any database path for a volume, they are accessible from the cluster through the default S3/object store segment. See segmentConfigurations below.
     */
    segmentConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxDataviewSegmentConfiguration>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

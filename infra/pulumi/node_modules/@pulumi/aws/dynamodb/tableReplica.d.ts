import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a DynamoDB table replica resource for [DynamoDB Global Tables V2 (version 2019.11.21)](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/globaltables.V2.html).
 *
 * > **Note:** Use `lifecycle` `ignoreChanges` for `replica` in the associated aws.dynamodb.Table configuration.
 *
 * > **Note:** Do not use the `replica` configuration block of aws.dynamodb.Table together with this resource as the two configuration options are mutually exclusive.
 *
 * ## Example Usage
 *
 * ### Basic Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dynamodb.Table("example", {
 *     name: "TestTable",
 *     hashKey: "BrodoBaggins",
 *     billingMode: "PAY_PER_REQUEST",
 *     streamEnabled: true,
 *     streamViewType: "NEW_AND_OLD_IMAGES",
 *     attributes: [{
 *         name: "BrodoBaggins",
 *         type: "S",
 *     }],
 * });
 * const exampleTableReplica = new aws.dynamodb.TableReplica("example", {
 *     globalTableArn: example.arn,
 *     tags: {
 *         Name: "IZPAWS",
 *         Pozo: "Amargo",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DynamoDB table replicas using the `table-name:main-region`. For example:
 *
 * ~> __Note:__ When importing, use the region where the initial or _main_ global table resides, _not_ the region of the replica.
 *
 * ```sh
 * $ pulumi import aws:dynamodb/tableReplica:TableReplica example TestTable:us-west-2
 * ```
 */
export declare class TableReplica extends pulumi.CustomResource {
    /**
     * Get an existing TableReplica resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TableReplicaState, opts?: pulumi.CustomResourceOptions): TableReplica;
    /**
     * Returns true if the given object is an instance of TableReplica.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TableReplica;
    /**
     * ARN of the table replica.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether deletion protection is enabled (true) or disabled (false) on the table replica.
     */
    readonly deletionProtectionEnabled: pulumi.Output<boolean>;
    /**
     * ARN of the _main_ or global table which this resource will replicate.
     *
     * The following arguments are optional:
     */
    readonly globalTableArn: pulumi.Output<string>;
    /**
     * ARN of the CMK that should be used for the AWS KMS encryption. This argument should only be used if the key is different from the default KMS-managed DynamoDB key, `alias/aws/dynamodb`. **Note:** This attribute will _not_ be populated with the ARN of _default_ keys.
     */
    readonly kmsKeyArn: pulumi.Output<string>;
    /**
     * Whether to enable Point In Time Recovery for the table replica. Default is `false`.
     */
    readonly pointInTimeRecovery: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Storage class of the table replica. Valid values are `STANDARD` and `STANDARD_INFREQUENT_ACCESS`. If not used, the table replica will use the same class as the global table.
     */
    readonly tableClassOverride: pulumi.Output<string | undefined>;
    /**
     * Map of tags to populate on the created table. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a TableReplica resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TableReplicaArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TableReplica resources.
 */
export interface TableReplicaState {
    /**
     * ARN of the table replica.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether deletion protection is enabled (true) or disabled (false) on the table replica.
     */
    deletionProtectionEnabled?: pulumi.Input<boolean>;
    /**
     * ARN of the _main_ or global table which this resource will replicate.
     *
     * The following arguments are optional:
     */
    globalTableArn?: pulumi.Input<string>;
    /**
     * ARN of the CMK that should be used for the AWS KMS encryption. This argument should only be used if the key is different from the default KMS-managed DynamoDB key, `alias/aws/dynamodb`. **Note:** This attribute will _not_ be populated with the ARN of _default_ keys.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Whether to enable Point In Time Recovery for the table replica. Default is `false`.
     */
    pointInTimeRecovery?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Storage class of the table replica. Valid values are `STANDARD` and `STANDARD_INFREQUENT_ACCESS`. If not used, the table replica will use the same class as the global table.
     */
    tableClassOverride?: pulumi.Input<string>;
    /**
     * Map of tags to populate on the created table. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a TableReplica resource.
 */
export interface TableReplicaArgs {
    /**
     * Whether deletion protection is enabled (true) or disabled (false) on the table replica.
     */
    deletionProtectionEnabled?: pulumi.Input<boolean>;
    /**
     * ARN of the _main_ or global table which this resource will replicate.
     *
     * The following arguments are optional:
     */
    globalTableArn: pulumi.Input<string>;
    /**
     * ARN of the CMK that should be used for the AWS KMS encryption. This argument should only be used if the key is different from the default KMS-managed DynamoDB key, `alias/aws/dynamodb`. **Note:** This attribute will _not_ be populated with the ARN of _default_ keys.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Whether to enable Point In Time Recovery for the table replica. Default is `false`.
     */
    pointInTimeRecovery?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Storage class of the table replica. Valid values are `STANDARD` and `STANDARD_INFREQUENT_ACCESS`. If not used, the table replica will use the same class as the global table.
     */
    tableClassOverride?: pulumi.Input<string>;
    /**
     * Map of tags to populate on the created table. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Redshift Data Share Consumer Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.DataShareConsumerAssociation("example", {
 *     dataShareArn: "arn:aws:redshift:us-west-2:123456789012:datashare:b3bfde75-73fd-408b-9086-d6fccfd6d588/example",
 *     associateEntireAccount: true,
 * });
 * ```
 *
 * ### Consumer Region
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.DataShareConsumerAssociation("example", {
 *     dataShareArn: "arn:aws:redshift:us-west-2:123456789012:datashare:b3bfde75-73fd-408b-9086-d6fccfd6d588/example",
 *     consumerRegion: "us-west-2",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Data Share Consumer Association using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/dataShareConsumerAssociation:DataShareConsumerAssociation example arn:aws:redshift:us-west-2:123456789012:datashare:b3bfde75-73fd-408b-9086-d6fccfd6d588/example,,,us-west-2
 * ```
 */
export declare class DataShareConsumerAssociation extends pulumi.CustomResource {
    /**
     * Get an existing DataShareConsumerAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataShareConsumerAssociationState, opts?: pulumi.CustomResourceOptions): DataShareConsumerAssociation;
    /**
     * Returns true if the given object is an instance of DataShareConsumerAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataShareConsumerAssociation;
    /**
     * Whether to allow write operations for a datashare.
     */
    readonly allowWrites: pulumi.Output<boolean | undefined>;
    /**
     * Whether the datashare is associated with the entire account. Conflicts with `consumerArn` and `consumerRegion`.
     */
    readonly associateEntireAccount: pulumi.Output<boolean | undefined>;
    /**
     * Amazon Resource Name (ARN) of the consumer that is associated with the datashare. Conflicts with `associateEntireAccount` and `consumerRegion`.
     */
    readonly consumerArn: pulumi.Output<string | undefined>;
    /**
     * From a datashare consumer account, associates a datashare with all existing and future namespaces in the specified AWS Region. Conflicts with `associateEntireAccount` and `consumerArn`.
     */
    readonly consumerRegion: pulumi.Output<string | undefined>;
    /**
     * Amazon Resource Name (ARN) of the datashare that the consumer is to use with the account or the namespace.
     *
     * The following arguments are optional:
     */
    readonly dataShareArn: pulumi.Output<string>;
    /**
     * Identifier of a datashare to show its managing entity.
     */
    readonly managedBy: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the producer.
     */
    readonly producerArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DataShareConsumerAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataShareConsumerAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataShareConsumerAssociation resources.
 */
export interface DataShareConsumerAssociationState {
    /**
     * Whether to allow write operations for a datashare.
     */
    allowWrites?: pulumi.Input<boolean>;
    /**
     * Whether the datashare is associated with the entire account. Conflicts with `consumerArn` and `consumerRegion`.
     */
    associateEntireAccount?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the consumer that is associated with the datashare. Conflicts with `associateEntireAccount` and `consumerRegion`.
     */
    consumerArn?: pulumi.Input<string>;
    /**
     * From a datashare consumer account, associates a datashare with all existing and future namespaces in the specified AWS Region. Conflicts with `associateEntireAccount` and `consumerArn`.
     */
    consumerRegion?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the datashare that the consumer is to use with the account or the namespace.
     *
     * The following arguments are optional:
     */
    dataShareArn?: pulumi.Input<string>;
    /**
     * Identifier of a datashare to show its managing entity.
     */
    managedBy?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the producer.
     */
    producerArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DataShareConsumerAssociation resource.
 */
export interface DataShareConsumerAssociationArgs {
    /**
     * Whether to allow write operations for a datashare.
     */
    allowWrites?: pulumi.Input<boolean>;
    /**
     * Whether the datashare is associated with the entire account. Conflicts with `consumerArn` and `consumerRegion`.
     */
    associateEntireAccount?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the consumer that is associated with the datashare. Conflicts with `associateEntireAccount` and `consumerRegion`.
     */
    consumerArn?: pulumi.Input<string>;
    /**
     * From a datashare consumer account, associates a datashare with all existing and future namespaces in the specified AWS Region. Conflicts with `associateEntireAccount` and `consumerArn`.
     */
    consumerRegion?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the datashare that the consumer is to use with the account or the namespace.
     *
     * The following arguments are optional:
     */
    dataShareArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

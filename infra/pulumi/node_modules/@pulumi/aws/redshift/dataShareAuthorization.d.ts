import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Redshift Data Share Authorization.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.DataShareAuthorization("example", {
 *     consumerIdentifier: "123456789012",
 *     dataShareArn: "arn:aws:redshift:us-west-2:123456789012:datashare:3072dae5-022b-4d45-9cd3-01f010aae4b2/example_share",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Data Share Authorization using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/dataShareAuthorization:DataShareAuthorization example arn:aws:redshift:us-west-2:123456789012:datashare:3072dae5-022b-4d45-9cd3-01f010aae4b2/example_share,123456789012
 * ```
 */
export declare class DataShareAuthorization extends pulumi.CustomResource {
    /**
     * Get an existing DataShareAuthorization resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataShareAuthorizationState, opts?: pulumi.CustomResourceOptions): DataShareAuthorization;
    /**
     * Returns true if the given object is an instance of DataShareAuthorization.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataShareAuthorization;
    /**
     * Whether to allow write operations for a datashare.
     */
    readonly allowWrites: pulumi.Output<boolean | undefined>;
    /**
     * Identifier of the data consumer that is authorized to access the datashare. This identifier is an AWS account ID or a keyword, such as `ADX`.
     */
    readonly consumerIdentifier: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the datashare that producers are to authorize sharing for.
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
     * Create a DataShareAuthorization resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataShareAuthorizationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataShareAuthorization resources.
 */
export interface DataShareAuthorizationState {
    /**
     * Whether to allow write operations for a datashare.
     */
    allowWrites?: pulumi.Input<boolean>;
    /**
     * Identifier of the data consumer that is authorized to access the datashare. This identifier is an AWS account ID or a keyword, such as `ADX`.
     */
    consumerIdentifier?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the datashare that producers are to authorize sharing for.
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
 * The set of arguments for constructing a DataShareAuthorization resource.
 */
export interface DataShareAuthorizationArgs {
    /**
     * Whether to allow write operations for a datashare.
     */
    allowWrites?: pulumi.Input<boolean>;
    /**
     * Identifier of the data consumer that is authorized to access the datashare. This identifier is an AWS account ID or a keyword, such as `ADX`.
     */
    consumerIdentifier: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the datashare that producers are to authorize sharing for.
     *
     * The following arguments are optional:
     */
    dataShareArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

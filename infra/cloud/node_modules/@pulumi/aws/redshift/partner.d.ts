import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a new Amazon Redshift Partner Integration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.Partner("example", {
 *     clusterIdentifier: exampleAwsRedshiftCluster.id,
 *     accountId: "1234567910",
 *     databaseName: exampleAwsRedshiftCluster.databaseName,
 *     partnerName: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift usage limits using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/partner:Partner example 01234567910:cluster-example-id:example:example
 * ```
 */
export declare class Partner extends pulumi.CustomResource {
    /**
     * Get an existing Partner resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PartnerState, opts?: pulumi.CustomResourceOptions): Partner;
    /**
     * Returns true if the given object is an instance of Partner.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Partner;
    /**
     * The Amazon Web Services account ID that owns the cluster.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The cluster identifier of the cluster that receives data from the partner.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The name of the database that receives data from the partner.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * The name of the partner that is authorized to send data.
     */
    readonly partnerName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * (Optional) The partner integration status.
     */
    readonly status: pulumi.Output<string>;
    /**
     * (Optional) The status message provided by the partner.
     */
    readonly statusMessage: pulumi.Output<string>;
    /**
     * Create a Partner resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PartnerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Partner resources.
 */
export interface PartnerState {
    /**
     * The Amazon Web Services account ID that owns the cluster.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The cluster identifier of the cluster that receives data from the partner.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The name of the database that receives data from the partner.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * The name of the partner that is authorized to send data.
     */
    partnerName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * (Optional) The partner integration status.
     */
    status?: pulumi.Input<string>;
    /**
     * (Optional) The status message provided by the partner.
     */
    statusMessage?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Partner resource.
 */
export interface PartnerArgs {
    /**
     * The Amazon Web Services account ID that owns the cluster.
     */
    accountId: pulumi.Input<string>;
    /**
     * The cluster identifier of the cluster that receives data from the partner.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The name of the database that receives data from the partner.
     */
    databaseName: pulumi.Input<string>;
    /**
     * The name of the partner that is authorized to send data.
     */
    partnerName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * Associates a single SCRAM secret with a Managed Streaming for Kafka (MSK) cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.msk.SingleScramSecretAssociation("example", {
 *     clusterArn: exampleAwsMskCluster.arn,
 *     secretArn: exampleAwsSecretsmanagerSecret.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an MSK SCRAM Secret Association using the `cluster_arn` and `secret_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:msk/singleScramSecretAssociation:SingleScramSecretAssociation example arn:aws:kafka:us-west-2:123456789012:cluster/example/279c0212-d057-4dba-9aa9-1c4e5a25bfc7-3,arn:aws:secretsmanager:us-east-1:123456789012:secret:example-123456
 * ```
 */
export declare class SingleScramSecretAssociation extends pulumi.CustomResource {
    /**
     * Get an existing SingleScramSecretAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SingleScramSecretAssociationState, opts?: pulumi.CustomResourceOptions): SingleScramSecretAssociation;
    /**
     * Returns true if the given object is an instance of SingleScramSecretAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SingleScramSecretAssociation;
    /**
     * Amazon Resource Name (ARN) of the MSK cluster.
     */
    readonly clusterArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * AWS Secrets Manager secret ARN.
     */
    readonly secretArn: pulumi.Output<string>;
    /**
     * Create a SingleScramSecretAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SingleScramSecretAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SingleScramSecretAssociation resources.
 */
export interface SingleScramSecretAssociationState {
    /**
     * Amazon Resource Name (ARN) of the MSK cluster.
     */
    clusterArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS Secrets Manager secret ARN.
     */
    secretArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SingleScramSecretAssociation resource.
 */
export interface SingleScramSecretAssociationArgs {
    /**
     * Amazon Resource Name (ARN) of the MSK cluster.
     */
    clusterArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS Secrets Manager secret ARN.
     */
    secretArn: pulumi.Input<string>;
}

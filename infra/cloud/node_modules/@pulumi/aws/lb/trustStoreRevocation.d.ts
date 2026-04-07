import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a ELBv2 Trust Store Revocation for use with Application Load Balancer Listener resources.
 *
 * ## Example Usage
 *
 * ### Trust Store With Revocations
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.lb.TrustStore("test", {
 *     name: "tf-example-lb-ts",
 *     caCertificatesBundleS3Bucket: "...",
 *     caCertificatesBundleS3Key: "...",
 * });
 * const testTrustStoreRevocation = new aws.lb.TrustStoreRevocation("test", {
 *     trustStoreArn: test.arn,
 *     revocationsS3Bucket: "...",
 *     revocationsS3Key: "...",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Trust Store Revocations using their ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:lb/trustStoreRevocation:TrustStoreRevocation example arn:aws:elasticloadbalancing:us-west-2:187416307283:truststore/my-trust-store/20cfe21448b66314,6
 * ```
 */
export declare class TrustStoreRevocation extends pulumi.CustomResource {
    /**
     * Get an existing TrustStoreRevocation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustStoreRevocationState, opts?: pulumi.CustomResourceOptions): TrustStoreRevocation;
    /**
     * Returns true if the given object is an instance of TrustStoreRevocation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustStoreRevocation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * AWS assigned RevocationId, (number).
     */
    readonly revocationId: pulumi.Output<number>;
    /**
     * S3 Bucket name holding the client certificate CA bundle.
     */
    readonly revocationsS3Bucket: pulumi.Output<string>;
    /**
     * S3 object key holding the client certificate CA bundle.
     */
    readonly revocationsS3Key: pulumi.Output<string>;
    /**
     * Version Id of CA bundle S3 bucket object, if versioned, defaults to latest if omitted.
     */
    readonly revocationsS3ObjectVersion: pulumi.Output<string | undefined>;
    /**
     * Trust Store ARN.
     */
    readonly trustStoreArn: pulumi.Output<string>;
    /**
     * Create a TrustStoreRevocation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustStoreRevocationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustStoreRevocation resources.
 */
export interface TrustStoreRevocationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS assigned RevocationId, (number).
     */
    revocationId?: pulumi.Input<number>;
    /**
     * S3 Bucket name holding the client certificate CA bundle.
     */
    revocationsS3Bucket?: pulumi.Input<string>;
    /**
     * S3 object key holding the client certificate CA bundle.
     */
    revocationsS3Key?: pulumi.Input<string>;
    /**
     * Version Id of CA bundle S3 bucket object, if versioned, defaults to latest if omitted.
     */
    revocationsS3ObjectVersion?: pulumi.Input<string>;
    /**
     * Trust Store ARN.
     */
    trustStoreArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TrustStoreRevocation resource.
 */
export interface TrustStoreRevocationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * S3 Bucket name holding the client certificate CA bundle.
     */
    revocationsS3Bucket: pulumi.Input<string>;
    /**
     * S3 object key holding the client certificate CA bundle.
     */
    revocationsS3Key: pulumi.Input<string>;
    /**
     * Version Id of CA bundle S3 bucket object, if versioned, defaults to latest if omitted.
     */
    revocationsS3ObjectVersion?: pulumi.Input<string>;
    /**
     * Trust Store ARN.
     */
    trustStoreArn: pulumi.Input<string>;
}

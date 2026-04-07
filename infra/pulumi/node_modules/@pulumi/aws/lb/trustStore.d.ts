import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a ELBv2 Trust Store for use with Application Load Balancer Listener resources.
 *
 * ## Example Usage
 *
 * ### Trust Store Load Balancer Listener
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
 * const example = new aws.lb.Listener("example", {
 *     loadBalancerArn: exampleAwsLb.id,
 *     defaultActions: [{
 *         targetGroupArn: exampleAwsLbTargetGroup.id,
 *         type: "forward",
 *     }],
 *     mutualAuthentication: {
 *         mode: "verify",
 *         trustStoreArn: test.arn,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the trust store.
 *
 * Using `pulumi import`, import Target Groups using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_lb_trust_store.example arn:aws:elasticloadbalancing:us-west-2:187416307283:truststore/my-trust-store/20cfe21448b66314
 */
export declare class TrustStore extends pulumi.CustomResource {
    /**
     * Get an existing TrustStore resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustStoreState, opts?: pulumi.CustomResourceOptions): TrustStore;
    /**
     * Returns true if the given object is an instance of TrustStore.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustStore;
    /**
     * ARN of the Trust Store (matches `id`).
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN suffix for use with CloudWatch Metrics.
     */
    readonly arnSuffix: pulumi.Output<string>;
    /**
     * S3 Bucket name holding the client certificate CA bundle.
     */
    readonly caCertificatesBundleS3Bucket: pulumi.Output<string>;
    /**
     * S3 object key holding the client certificate CA bundle.
     */
    readonly caCertificatesBundleS3Key: pulumi.Output<string>;
    /**
     * Version Id of CA bundle S3 bucket object, if versioned, defaults to latest if omitted.
     */
    readonly caCertificatesBundleS3ObjectVersion: pulumi.Output<string | undefined>;
    /**
     * Name of the Trust Store. If omitted, the provider will assign a random, unique name. This name must be unique per region per account, can have a maximum of 32 characters, must contain only alphanumeric characters or hyphens, and must not begin or end with a hyphen.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`. Cannot be longer than 6 characters.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a TrustStore resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustStoreArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustStore resources.
 */
export interface TrustStoreState {
    /**
     * ARN of the Trust Store (matches `id`).
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN suffix for use with CloudWatch Metrics.
     */
    arnSuffix?: pulumi.Input<string>;
    /**
     * S3 Bucket name holding the client certificate CA bundle.
     */
    caCertificatesBundleS3Bucket?: pulumi.Input<string>;
    /**
     * S3 object key holding the client certificate CA bundle.
     */
    caCertificatesBundleS3Key?: pulumi.Input<string>;
    /**
     * Version Id of CA bundle S3 bucket object, if versioned, defaults to latest if omitted.
     */
    caCertificatesBundleS3ObjectVersion?: pulumi.Input<string>;
    /**
     * Name of the Trust Store. If omitted, the provider will assign a random, unique name. This name must be unique per region per account, can have a maximum of 32 characters, must contain only alphanumeric characters or hyphens, and must not begin or end with a hyphen.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`. Cannot be longer than 6 characters.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a TrustStore resource.
 */
export interface TrustStoreArgs {
    /**
     * S3 Bucket name holding the client certificate CA bundle.
     */
    caCertificatesBundleS3Bucket: pulumi.Input<string>;
    /**
     * S3 object key holding the client certificate CA bundle.
     */
    caCertificatesBundleS3Key: pulumi.Input<string>;
    /**
     * Version Id of CA bundle S3 bucket object, if versioned, defaults to latest if omitted.
     */
    caCertificatesBundleS3ObjectVersion?: pulumi.Input<string>;
    /**
     * Name of the Trust Store. If omitted, the provider will assign a random, unique name. This name must be unique per region per account, can have a maximum of 32 characters, must contain only alphanumeric characters or hyphens, and must not begin or end with a hyphen.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`. Cannot be longer than 6 characters.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

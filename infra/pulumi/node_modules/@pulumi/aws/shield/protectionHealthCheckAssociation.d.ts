import * as pulumi from "@pulumi/pulumi";
/**
 * Creates an association between a Route53 Health Check and a Shield Advanced protected resource.
 * This association uses the health of your applications to improve responsiveness and accuracy in attack detection and mitigation.
 *
 * Blog post: [AWS Shield Advanced now supports Health Based Detection](https://aws.amazon.com/about-aws/whats-new/2020/02/aws-shield-advanced-now-supports-health-based-detection/)
 *
 * ## Example Usage
 *
 * ### Create an association between a protected EIP and a Route53 Health Check
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const example = new aws.ec2.Eip("example", {
 *     domain: "vpc",
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * const exampleProtection = new aws.shield.Protection("example", {
 *     name: "example-protection",
 *     resourceArn: pulumi.all([currentGetPartition, current, currentGetCallerIdentity, example.id]).apply(([currentGetPartition, current, currentGetCallerIdentity, id]) => `arn:${currentGetPartition.partition}:ec2:${current.region}:${currentGetCallerIdentity.accountId}:eip-allocation/${id}`),
 * });
 * const exampleHealthCheck = new aws.route53.HealthCheck("example", {
 *     ipAddress: example.publicIp,
 *     port: 80,
 *     type: "HTTP",
 *     resourcePath: "/ready",
 *     failureThreshold: 3,
 *     requestInterval: 30,
 *     tags: {
 *         Name: "tf-example-health-check",
 *     },
 * });
 * const exampleProtectionHealthCheckAssociation = new aws.shield.ProtectionHealthCheckAssociation("example", {
 *     healthCheckArn: exampleHealthCheck.arn,
 *     shieldProtectionId: exampleProtection.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Shield protection health check association resources using the `shield_protection_id` and `health_check_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:shield/protectionHealthCheckAssociation:ProtectionHealthCheckAssociation example ff9592dc-22f3-4e88-afa1-7b29fde9669a+arn:aws:route53:::healthcheck/3742b175-edb9-46bc-9359-f53e3b794b1b
 * ```
 */
export declare class ProtectionHealthCheckAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ProtectionHealthCheckAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProtectionHealthCheckAssociationState, opts?: pulumi.CustomResourceOptions): ProtectionHealthCheckAssociation;
    /**
     * Returns true if the given object is an instance of ProtectionHealthCheckAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProtectionHealthCheckAssociation;
    /**
     * The ARN (Amazon Resource Name) of the Route53 Health Check resource which will be associated to the protected resource.
     */
    readonly healthCheckArn: pulumi.Output<string>;
    /**
     * The ID of the protected resource.
     */
    readonly shieldProtectionId: pulumi.Output<string>;
    /**
     * Create a ProtectionHealthCheckAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProtectionHealthCheckAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProtectionHealthCheckAssociation resources.
 */
export interface ProtectionHealthCheckAssociationState {
    /**
     * The ARN (Amazon Resource Name) of the Route53 Health Check resource which will be associated to the protected resource.
     */
    healthCheckArn?: pulumi.Input<string>;
    /**
     * The ID of the protected resource.
     */
    shieldProtectionId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ProtectionHealthCheckAssociation resource.
 */
export interface ProtectionHealthCheckAssociationArgs {
    /**
     * The ARN (Amazon Resource Name) of the Route53 Health Check resource which will be associated to the protected resource.
     */
    healthCheckArn: pulumi.Input<string>;
    /**
     * The ID of the protected resource.
     */
    shieldProtectionId: pulumi.Input<string>;
}

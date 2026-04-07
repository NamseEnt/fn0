import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a License Manager association.
 *
 * > **Note:** License configurations can also be associated with launch templates by specifying the `licenseSpecifications` block for an `aws.ec2.LaunchTemplate`.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getAmi({
 *     mostRecent: true,
 *     owners: ["amazon"],
 *     filters: [{
 *         name: "name",
 *         values: ["amzn-ami-vpc-nat*"],
 *     }],
 * });
 * const exampleInstance = new aws.ec2.Instance("example", {
 *     ami: example.then(example => example.id),
 *     instanceType: aws.ec2.InstanceType.T2_Micro,
 * });
 * const exampleLicenseConfiguration = new aws.licensemanager.LicenseConfiguration("example", {
 *     name: "Example",
 *     licenseCountingType: "Instance",
 * });
 * const exampleAssociation = new aws.licensemanager.Association("example", {
 *     licenseConfigurationArn: exampleLicenseConfiguration.arn,
 *     resourceArn: exampleInstance.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import license configurations using `resource_arn,license_configuration_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:licensemanager/association:Association example arn:aws:ec2:eu-west-1:123456789012:image/ami-123456789abcdef01,arn:aws:license-manager:eu-west-1:123456789012:license-configuration:lic-0123456789abcdef0123456789abcdef
 * ```
 */
export declare class Association extends pulumi.CustomResource {
    /**
     * Get an existing Association resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AssociationState, opts?: pulumi.CustomResourceOptions): Association;
    /**
     * Returns true if the given object is an instance of Association.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Association;
    /**
     * ARN of the license configuration.
     */
    readonly licenseConfigurationArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the resource associated with the license configuration.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a Association resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Association resources.
 */
export interface AssociationState {
    /**
     * ARN of the license configuration.
     */
    licenseConfigurationArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the resource associated with the license configuration.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Association resource.
 */
export interface AssociationArgs {
    /**
     * ARN of the license configuration.
     */
    licenseConfigurationArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the resource associated with the license configuration.
     */
    resourceArn: pulumi.Input<string>;
}

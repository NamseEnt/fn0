import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS WorkSpaces Web Network Settings Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["opt-in-not-required"],
 *     }],
 * });
 * const example = new aws.ec2.Vpc("example", {
 *     cidrBlock: "10.0.0.0/16",
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * const exampleSubnet: aws.ec2.Subnet[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     exampleSubnet.push(new aws.ec2.Subnet(`example-${range.value}`, {
 *         vpcId: example.id,
 *         cidrBlock: example.cidrBlock.apply(cidrBlock => std.cidrsubnetOutput({
 *             input: cidrBlock,
 *             newbits: 8,
 *             netnum: range.value,
 *         })).apply(invoke => invoke.result),
 *         availabilityZone: available.then(available => available.names[range.value]),
 *         tags: {
 *             Name: "example",
 *         },
 *     }));
 * }
 * const exampleSecurityGroup: aws.ec2.SecurityGroup[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     exampleSecurityGroup.push(new aws.ec2.SecurityGroup(`example-${range.value}`, {
 *         vpcId: example.id,
 *         name: `example-${range.value}`,
 *         tags: {
 *             Name: "example",
 *         },
 *     }));
 * }
 * const examplePortal = new aws.workspacesweb.Portal("example", {displayName: "example"});
 * const exampleNetworkSettings = new aws.workspacesweb.NetworkSettings("example", {
 *     vpcId: example.id,
 *     subnetIds: [
 *         exampleSubnet[0].id,
 *         exampleSubnet[1].id,
 *     ],
 *     securityGroupIds: [
 *         exampleSecurityGroup[0].id,
 *         exampleSecurityGroup[1].id,
 *     ],
 * });
 * const exampleNetworkSettingsAssociation = new aws.workspacesweb.NetworkSettingsAssociation("example", {
 *     networkSettingsArn: exampleNetworkSettings.networkSettingsArn,
 *     portalArn: examplePortal.portalArn,
 * });
 * ```
 */
export declare class NetworkSettingsAssociation extends pulumi.CustomResource {
    /**
     * Get an existing NetworkSettingsAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkSettingsAssociationState, opts?: pulumi.CustomResourceOptions): NetworkSettingsAssociation;
    /**
     * Returns true if the given object is an instance of NetworkSettingsAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkSettingsAssociation;
    /**
     * ARN of the network settings to associate with the portal. Forces replacement if changed.
     */
    readonly networkSettingsArn: pulumi.Output<string>;
    /**
     * ARN of the portal to associate with the network settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    readonly portalArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a NetworkSettingsAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkSettingsAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkSettingsAssociation resources.
 */
export interface NetworkSettingsAssociationState {
    /**
     * ARN of the network settings to associate with the portal. Forces replacement if changed.
     */
    networkSettingsArn?: pulumi.Input<string>;
    /**
     * ARN of the portal to associate with the network settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NetworkSettingsAssociation resource.
 */
export interface NetworkSettingsAssociationArgs {
    /**
     * ARN of the network settings to associate with the portal. Forces replacement if changed.
     */
    networkSettingsArn: pulumi.Input<string>;
    /**
     * ARN of the portal to associate with the network settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

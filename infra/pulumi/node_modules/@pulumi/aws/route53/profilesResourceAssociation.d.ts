import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Route 53 Profiles Resource Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53.ProfilesProfile("example", {name: "example"});
 * const exampleVpc = new aws.ec2.Vpc("example", {cidr: "10.0.0.0/16"});
 * const exampleZone = new aws.route53.Zone("example", {
 *     name: "example.com",
 *     vpcs: [{
 *         vpcId: exampleVpc.id,
 *     }],
 * });
 * const exampleProfilesResourceAssociation = new aws.route53.ProfilesResourceAssociation("example", {
 *     name: "example",
 *     profileId: example.id,
 *     resourceArn: exampleZone.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route 53 Profiles Resource Association using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/profilesResourceAssociation:ProfilesResourceAssociation example rpa-id-12345678
 * ```
 */
export declare class ProfilesResourceAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ProfilesResourceAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProfilesResourceAssociationState, opts?: pulumi.CustomResourceOptions): ProfilesResourceAssociation;
    /**
     * Returns true if the given object is an instance of ProfilesResourceAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProfilesResourceAssociation;
    /**
     * Name of the Profile Resource Association.
     */
    readonly name: pulumi.Output<string>;
    readonly ownerId: pulumi.Output<string>;
    /**
     * ID of the profile associated with the VPC.
     */
    readonly profileId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Resource ID of the resource to be associated with the profile.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Resource properties for the resource to be associated with the profile.
     */
    readonly resourceProperties: pulumi.Output<string>;
    /**
     * Type of resource associated with the profile.
     */
    readonly resourceType: pulumi.Output<string>;
    /**
     * Status of the Profile Association. Valid values [AWS docs](https://docs.aws.amazon.com/Route53/latest/APIReference/API_route53profiles_Profile.html)
     */
    readonly status: pulumi.Output<string>;
    /**
     * Status message of the Profile Resource Association.
     */
    readonly statusMessage: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.route53.ProfilesResourceAssociationTimeouts | undefined>;
    /**
     * Create a ProfilesResourceAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProfilesResourceAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProfilesResourceAssociation resources.
 */
export interface ProfilesResourceAssociationState {
    /**
     * Name of the Profile Resource Association.
     */
    name?: pulumi.Input<string>;
    ownerId?: pulumi.Input<string>;
    /**
     * ID of the profile associated with the VPC.
     */
    profileId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Resource ID of the resource to be associated with the profile.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Resource properties for the resource to be associated with the profile.
     */
    resourceProperties?: pulumi.Input<string>;
    /**
     * Type of resource associated with the profile.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Status of the Profile Association. Valid values [AWS docs](https://docs.aws.amazon.com/Route53/latest/APIReference/API_route53profiles_Profile.html)
     */
    status?: pulumi.Input<string>;
    /**
     * Status message of the Profile Resource Association.
     */
    statusMessage?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.route53.ProfilesResourceAssociationTimeouts>;
}
/**
 * The set of arguments for constructing a ProfilesResourceAssociation resource.
 */
export interface ProfilesResourceAssociationArgs {
    /**
     * Name of the Profile Resource Association.
     */
    name?: pulumi.Input<string>;
    /**
     * ID of the profile associated with the VPC.
     */
    profileId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Resource ID of the resource to be associated with the profile.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * Resource properties for the resource to be associated with the profile.
     */
    resourceProperties?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.route53.ProfilesResourceAssociationTimeouts>;
}

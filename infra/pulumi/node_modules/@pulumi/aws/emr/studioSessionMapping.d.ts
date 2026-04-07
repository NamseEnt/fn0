import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic MapReduce Studio Session Mapping.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emr.StudioSessionMapping("example", {
 *     studioId: exampleAwsEmrStudio.id,
 *     identityType: "USER",
 *     identityId: "example",
 *     sessionPolicyArn: exampleAwsIamPolicy.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EMR studio session mappings using `studio-id:identity-type:identity-id`. For example:
 *
 * ```sh
 * $ pulumi import aws:emr/studioSessionMapping:StudioSessionMapping example es-xxxxx:USER:xxxxx-xxx-xxx
 * ```
 */
export declare class StudioSessionMapping extends pulumi.CustomResource {
    /**
     * Get an existing StudioSessionMapping resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StudioSessionMappingState, opts?: pulumi.CustomResourceOptions): StudioSessionMapping;
    /**
     * Returns true if the given object is an instance of StudioSessionMapping.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StudioSessionMapping;
    /**
     * The globally unique identifier (GUID) of the user or group from the Amazon Web Services SSO Identity Store.
     */
    readonly identityId: pulumi.Output<string>;
    /**
     * The name of the user or group from the Amazon Web Services SSO Identity Store.
     */
    readonly identityName: pulumi.Output<string>;
    /**
     * Specifies whether the identity to map to the Amazon EMR Studio is a `USER` or a `GROUP`.
     */
    readonly identityType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) for the session policy that will be applied to the user or group. You should specify the ARN for the session policy that you want to apply, not the ARN of your user role.
     */
    readonly sessionPolicyArn: pulumi.Output<string>;
    /**
     * The ID of the Amazon EMR Studio to which the user or group will be mapped.
     */
    readonly studioId: pulumi.Output<string>;
    /**
     * Create a StudioSessionMapping resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StudioSessionMappingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StudioSessionMapping resources.
 */
export interface StudioSessionMappingState {
    /**
     * The globally unique identifier (GUID) of the user or group from the Amazon Web Services SSO Identity Store.
     */
    identityId?: pulumi.Input<string>;
    /**
     * The name of the user or group from the Amazon Web Services SSO Identity Store.
     */
    identityName?: pulumi.Input<string>;
    /**
     * Specifies whether the identity to map to the Amazon EMR Studio is a `USER` or a `GROUP`.
     */
    identityType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the session policy that will be applied to the user or group. You should specify the ARN for the session policy that you want to apply, not the ARN of your user role.
     */
    sessionPolicyArn?: pulumi.Input<string>;
    /**
     * The ID of the Amazon EMR Studio to which the user or group will be mapped.
     */
    studioId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StudioSessionMapping resource.
 */
export interface StudioSessionMappingArgs {
    /**
     * The globally unique identifier (GUID) of the user or group from the Amazon Web Services SSO Identity Store.
     */
    identityId?: pulumi.Input<string>;
    /**
     * The name of the user or group from the Amazon Web Services SSO Identity Store.
     */
    identityName?: pulumi.Input<string>;
    /**
     * Specifies whether the identity to map to the Amazon EMR Studio is a `USER` or a `GROUP`.
     */
    identityType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the session policy that will be applied to the user or group. You should specify the ARN for the session policy that you want to apply, not the ARN of your user role.
     */
    sessionPolicyArn: pulumi.Input<string>;
    /**
     * The ID of the Amazon EMR Studio to which the user or group will be mapped.
     */
    studioId: pulumi.Input<string>;
}

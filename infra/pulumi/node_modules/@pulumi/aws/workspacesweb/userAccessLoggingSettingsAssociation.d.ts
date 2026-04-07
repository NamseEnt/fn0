import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS WorkSpaces Web User Access Logging Settings Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.workspacesweb.Portal("example", {displayName: "example"});
 * const exampleStream = new aws.kinesis.Stream("example", {
 *     name: "amazon-workspaces-web-example",
 *     shardCount: 1,
 * });
 * const exampleUserAccessLoggingSettings = new aws.workspacesweb.UserAccessLoggingSettings("example", {kinesisStreamArn: exampleStream.arn});
 * const exampleUserAccessLoggingSettingsAssociation = new aws.workspacesweb.UserAccessLoggingSettingsAssociation("example", {
 *     userAccessLoggingSettingsArn: exampleUserAccessLoggingSettings.userAccessLoggingSettingsArn,
 *     portalArn: example.portalArn,
 * });
 * ```
 */
export declare class UserAccessLoggingSettingsAssociation extends pulumi.CustomResource {
    /**
     * Get an existing UserAccessLoggingSettingsAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserAccessLoggingSettingsAssociationState, opts?: pulumi.CustomResourceOptions): UserAccessLoggingSettingsAssociation;
    /**
     * Returns true if the given object is an instance of UserAccessLoggingSettingsAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserAccessLoggingSettingsAssociation;
    /**
     * ARN of the portal to associate with the user access logging settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    readonly portalArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the user access logging settings to associate with the portal. Forces replacement if changed.
     */
    readonly userAccessLoggingSettingsArn: pulumi.Output<string>;
    /**
     * Create a UserAccessLoggingSettingsAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserAccessLoggingSettingsAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserAccessLoggingSettingsAssociation resources.
 */
export interface UserAccessLoggingSettingsAssociationState {
    /**
     * ARN of the portal to associate with the user access logging settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the user access logging settings to associate with the portal. Forces replacement if changed.
     */
    userAccessLoggingSettingsArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserAccessLoggingSettingsAssociation resource.
 */
export interface UserAccessLoggingSettingsAssociationArgs {
    /**
     * ARN of the portal to associate with the user access logging settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the user access logging settings to associate with the portal. Forces replacement if changed.
     */
    userAccessLoggingSettingsArn: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS WorkSpaces Web Data Protection Settings Association.
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
 * const exampleDataProtectionSettings = new aws.workspacesweb.DataProtectionSettings("example", {displayName: "example"});
 * const exampleDataProtectionSettingsAssociation = new aws.workspacesweb.DataProtectionSettingsAssociation("example", {
 *     dataProtectionSettingsArn: exampleDataProtectionSettings.dataProtectionSettingsArn,
 *     portalArn: example.portalArn,
 * });
 * ```
 */
export declare class DataProtectionSettingsAssociation extends pulumi.CustomResource {
    /**
     * Get an existing DataProtectionSettingsAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataProtectionSettingsAssociationState, opts?: pulumi.CustomResourceOptions): DataProtectionSettingsAssociation;
    /**
     * Returns true if the given object is an instance of DataProtectionSettingsAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataProtectionSettingsAssociation;
    /**
     * ARN of the data protection settings to associate with the portal. Forces replacement if changed.
     */
    readonly dataProtectionSettingsArn: pulumi.Output<string>;
    /**
     * ARN of the portal to associate with the data protection settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    readonly portalArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DataProtectionSettingsAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataProtectionSettingsAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataProtectionSettingsAssociation resources.
 */
export interface DataProtectionSettingsAssociationState {
    /**
     * ARN of the data protection settings to associate with the portal. Forces replacement if changed.
     */
    dataProtectionSettingsArn?: pulumi.Input<string>;
    /**
     * ARN of the portal to associate with the data protection settings. Forces replacement if changed.
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
 * The set of arguments for constructing a DataProtectionSettingsAssociation resource.
 */
export interface DataProtectionSettingsAssociationArgs {
    /**
     * ARN of the data protection settings to associate with the portal. Forces replacement if changed.
     */
    dataProtectionSettingsArn: pulumi.Input<string>;
    /**
     * ARN of the portal to associate with the data protection settings. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

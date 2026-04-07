import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS QuickSight Folder Membership.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.FolderMembership("example", {
 *     folderId: exampleAwsQuicksightFolder.folderId,
 *     memberType: "DATASET",
 *     memberId: exampleAwsQuicksightDataSet.dataSetId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight Folder Membership using the AWS account ID, folder ID, member type, and member ID separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/folderMembership:FolderMembership example 123456789012,example-folder,DATASET,example-dataset
 * ```
 */
export declare class FolderMembership extends pulumi.CustomResource {
    /**
     * Get an existing FolderMembership resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FolderMembershipState, opts?: pulumi.CustomResourceOptions): FolderMembership;
    /**
     * Returns true if the given object is an instance of FolderMembership.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FolderMembership;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Identifier for the folder.
     */
    readonly folderId: pulumi.Output<string>;
    /**
     * ID of the asset (the dashboard, analysis, or dataset).
     */
    readonly memberId: pulumi.Output<string>;
    /**
     * Type of the member. Valid values are `ANALYSIS`, `DASHBOARD`, and `DATASET`.
     *
     * The following arguments are optional:
     */
    readonly memberType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a FolderMembership resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FolderMembershipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FolderMembership resources.
 */
export interface FolderMembershipState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Identifier for the folder.
     */
    folderId?: pulumi.Input<string>;
    /**
     * ID of the asset (the dashboard, analysis, or dataset).
     */
    memberId?: pulumi.Input<string>;
    /**
     * Type of the member. Valid values are `ANALYSIS`, `DASHBOARD`, and `DATASET`.
     *
     * The following arguments are optional:
     */
    memberType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FolderMembership resource.
 */
export interface FolderMembershipArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Identifier for the folder.
     */
    folderId: pulumi.Input<string>;
    /**
     * ID of the asset (the dashboard, analysis, or dataset).
     */
    memberId: pulumi.Input<string>;
    /**
     * Type of the member. Valid values are `ANALYSIS`, `DASHBOARD`, and `DATASET`.
     *
     * The following arguments are optional:
     */
    memberType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * Accepts a shared directory in a consumer account.
 *
 * > **NOTE:** Destroying this resource removes the shared directory from the consumer account only.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directoryservice.SharedDirectory("example", {
 *     directoryId: exampleAwsDirectoryServiceDirectory.id,
 *     notes: "example",
 *     target: {
 *         id: receiver.accountId,
 *     },
 * });
 * const exampleSharedDirectoryAccepter = new aws.directoryservice.SharedDirectoryAccepter("example", {sharedDirectoryId: example.sharedDirectoryId});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Directory Service Shared Directories using the shared directory ID. For example:
 *
 * ```sh
 * $ pulumi import aws:directoryservice/sharedDirectoryAccepter:SharedDirectoryAccepter example d-9267633ece
 * ```
 */
export declare class SharedDirectoryAccepter extends pulumi.CustomResource {
    /**
     * Get an existing SharedDirectoryAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SharedDirectoryAccepterState, opts?: pulumi.CustomResourceOptions): SharedDirectoryAccepter;
    /**
     * Returns true if the given object is an instance of SharedDirectoryAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SharedDirectoryAccepter;
    /**
     * Method used when sharing a directory (i.e., `ORGANIZATIONS` or `HANDSHAKE`).
     */
    readonly method: pulumi.Output<string>;
    /**
     * Message sent by the directory owner to the directory consumer to help the directory consumer administrator determine whether to approve or reject the share invitation.
     */
    readonly notes: pulumi.Output<string>;
    /**
     * Account identifier of the directory owner.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * Identifier of the Managed Microsoft AD directory from the perspective of the directory owner.
     */
    readonly ownerDirectoryId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the directory that is stored in the directory consumer account that corresponds to the shared directory in the owner account.
     */
    readonly sharedDirectoryId: pulumi.Output<string>;
    /**
     * Create a SharedDirectoryAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SharedDirectoryAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SharedDirectoryAccepter resources.
 */
export interface SharedDirectoryAccepterState {
    /**
     * Method used when sharing a directory (i.e., `ORGANIZATIONS` or `HANDSHAKE`).
     */
    method?: pulumi.Input<string>;
    /**
     * Message sent by the directory owner to the directory consumer to help the directory consumer administrator determine whether to approve or reject the share invitation.
     */
    notes?: pulumi.Input<string>;
    /**
     * Account identifier of the directory owner.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * Identifier of the Managed Microsoft AD directory from the perspective of the directory owner.
     */
    ownerDirectoryId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the directory that is stored in the directory consumer account that corresponds to the shared directory in the owner account.
     */
    sharedDirectoryId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SharedDirectoryAccepter resource.
 */
export interface SharedDirectoryAccepterArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the directory that is stored in the directory consumer account that corresponds to the shared directory in the owner account.
     */
    sharedDirectoryId: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a directory in your account (directory owner) shared with another account (directory consumer).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directoryservice.Directory("example", {
 *     name: "tf-example",
 *     password: "SuperSecretPassw0rd",
 *     type: "MicrosoftAD",
 *     edition: "Standard",
 *     vpcSettings: {
 *         vpcId: exampleAwsVpc.id,
 *         subnetIds: exampleAwsSubnet.map(__item => __item.id),
 *     },
 * });
 * const exampleSharedDirectory = new aws.directoryservice.SharedDirectory("example", {
 *     directoryId: example.id,
 *     notes: "You wanna have a catch?",
 *     target: {
 *         id: receiver.accountId,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Directory Service Shared Directories using the owner directory ID/shared directory ID. For example:
 *
 * ```sh
 * $ pulumi import aws:directoryservice/sharedDirectory:SharedDirectory example d-1234567890/d-9267633ece
 * ```
 */
export declare class SharedDirectory extends pulumi.CustomResource {
    /**
     * Get an existing SharedDirectory resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SharedDirectoryState, opts?: pulumi.CustomResourceOptions): SharedDirectory;
    /**
     * Returns true if the given object is an instance of SharedDirectory.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SharedDirectory;
    /**
     * Identifier of the Managed Microsoft AD directory that you want to share with other accounts.
     */
    readonly directoryId: pulumi.Output<string>;
    /**
     * Method used when sharing a directory. Valid values are `ORGANIZATIONS` and `HANDSHAKE`. Default is `HANDSHAKE`.
     */
    readonly method: pulumi.Output<string | undefined>;
    /**
     * Message sent by the directory owner to the directory consumer to help the directory consumer administrator determine whether to approve or reject the share invitation.
     */
    readonly notes: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the directory that is stored in the directory consumer account that corresponds to the shared directory in the owner account.
     */
    readonly sharedDirectoryId: pulumi.Output<string>;
    /**
     * Identifier for the directory consumer account with whom the directory is to be shared. See below.
     *
     * The following arguments are optional:
     */
    readonly target: pulumi.Output<outputs.directoryservice.SharedDirectoryTarget>;
    /**
     * Create a SharedDirectory resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SharedDirectoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SharedDirectory resources.
 */
export interface SharedDirectoryState {
    /**
     * Identifier of the Managed Microsoft AD directory that you want to share with other accounts.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Method used when sharing a directory. Valid values are `ORGANIZATIONS` and `HANDSHAKE`. Default is `HANDSHAKE`.
     */
    method?: pulumi.Input<string>;
    /**
     * Message sent by the directory owner to the directory consumer to help the directory consumer administrator determine whether to approve or reject the share invitation.
     */
    notes?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the directory that is stored in the directory consumer account that corresponds to the shared directory in the owner account.
     */
    sharedDirectoryId?: pulumi.Input<string>;
    /**
     * Identifier for the directory consumer account with whom the directory is to be shared. See below.
     *
     * The following arguments are optional:
     */
    target?: pulumi.Input<inputs.directoryservice.SharedDirectoryTarget>;
}
/**
 * The set of arguments for constructing a SharedDirectory resource.
 */
export interface SharedDirectoryArgs {
    /**
     * Identifier of the Managed Microsoft AD directory that you want to share with other accounts.
     */
    directoryId: pulumi.Input<string>;
    /**
     * Method used when sharing a directory. Valid values are `ORGANIZATIONS` and `HANDSHAKE`. Default is `HANDSHAKE`.
     */
    method?: pulumi.Input<string>;
    /**
     * Message sent by the directory owner to the directory consumer to help the directory consumer administrator determine whether to approve or reject the share invitation.
     */
    notes?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier for the directory consumer account with whom the directory is to be shared. See below.
     *
     * The following arguments are optional:
     */
    target: pulumi.Input<inputs.directoryservice.SharedDirectoryTarget>;
}

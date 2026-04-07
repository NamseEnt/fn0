import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Audit Manager Framework Share.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.auditmanager.FrameworkShare("example", {
 *     destinationAccount: "123456789012",
 *     destinationRegion: "us-east-1",
 *     frameworkId: exampleAwsAuditmanagerFramework.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Audit Manager Framework Share using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:auditmanager/frameworkShare:FrameworkShare example abcdef-123456
 * ```
 */
export declare class FrameworkShare extends pulumi.CustomResource {
    /**
     * Get an existing FrameworkShare resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FrameworkShareState, opts?: pulumi.CustomResourceOptions): FrameworkShare;
    /**
     * Returns true if the given object is an instance of FrameworkShare.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FrameworkShare;
    /**
     * Comment from the sender about the share request.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * Amazon Web Services account of the recipient.
     */
    readonly destinationAccount: pulumi.Output<string>;
    /**
     * Amazon Web Services region of the recipient.
     */
    readonly destinationRegion: pulumi.Output<string>;
    /**
     * Unique identifier for the shared custom framework.
     *
     * The following arguments are optional:
     */
    readonly frameworkId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the share request.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a FrameworkShare resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FrameworkShareArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FrameworkShare resources.
 */
export interface FrameworkShareState {
    /**
     * Comment from the sender about the share request.
     */
    comment?: pulumi.Input<string>;
    /**
     * Amazon Web Services account of the recipient.
     */
    destinationAccount?: pulumi.Input<string>;
    /**
     * Amazon Web Services region of the recipient.
     */
    destinationRegion?: pulumi.Input<string>;
    /**
     * Unique identifier for the shared custom framework.
     *
     * The following arguments are optional:
     */
    frameworkId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the share request.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FrameworkShare resource.
 */
export interface FrameworkShareArgs {
    /**
     * Comment from the sender about the share request.
     */
    comment?: pulumi.Input<string>;
    /**
     * Amazon Web Services account of the recipient.
     */
    destinationAccount: pulumi.Input<string>;
    /**
     * Amazon Web Services region of the recipient.
     */
    destinationRegion: pulumi.Input<string>;
    /**
     * Unique identifier for the shared custom framework.
     *
     * The following arguments are optional:
     */
    frameworkId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

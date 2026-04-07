import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Attaches a permissions boundary policy to a Single Sign-On (SSO) Permission Set resource.
 *
 * > **NOTE:** A permission set can have at most one permissions boundary attached; using more than one `aws.ssoadmin.PermissionsBoundaryAttachment` references the same permission set will show a permanent difference.
 *
 * ## Example Usage
 *
 * ### Attaching a customer-managed policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const examplePermissionSet = new aws.ssoadmin.PermissionSet("example", {
 *     name: "Example",
 *     instanceArn: example.then(example => example.arns?.[0]),
 * });
 * const examplePolicy = new aws.iam.Policy("example", {
 *     name: "TestPolicy",
 *     description: "My test policy",
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: ["ec2:Describe*"],
 *             Effect: "Allow",
 *             Resource: "*",
 *         }],
 *     }),
 * });
 * const examplePermissionsBoundaryAttachment = new aws.ssoadmin.PermissionsBoundaryAttachment("example", {
 *     instanceArn: examplePermissionSet.instanceArn,
 *     permissionSetArn: examplePermissionSet.arn,
 *     permissionsBoundary: {
 *         customerManagedPolicyReference: {
 *             name: examplePolicy.name,
 *             path: "/",
 *         },
 *     },
 * });
 * ```
 *
 * ### Attaching an AWS-managed policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssoadmin.PermissionsBoundaryAttachment("example", {
 *     instanceArn: exampleAwsSsoadminPermissionSet.instanceArn,
 *     permissionSetArn: exampleAwsSsoadminPermissionSet.arn,
 *     permissionsBoundary: {
 *         managedPolicyArn: "arn:aws:iam::aws:policy/ReadOnlyAccess",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Admin Permissions Boundary Attachments using the `permission_set_arn` and `instance_arn`, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/permissionsBoundaryAttachment:PermissionsBoundaryAttachment example arn:aws:sso:::permissionSet/ssoins-2938j0x8920sbj72/ps-80383020jr9302rk,arn:aws:sso:::instance/ssoins-2938j0x8920sbj72
 * ```
 */
export declare class PermissionsBoundaryAttachment extends pulumi.CustomResource {
    /**
     * Get an existing PermissionsBoundaryAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PermissionsBoundaryAttachmentState, opts?: pulumi.CustomResourceOptions): PermissionsBoundaryAttachment;
    /**
     * Returns true if the given object is an instance of PermissionsBoundaryAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PermissionsBoundaryAttachment;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    readonly permissionSetArn: pulumi.Output<string>;
    /**
     * The permissions boundary policy. See below.
     */
    readonly permissionsBoundary: pulumi.Output<outputs.ssoadmin.PermissionsBoundaryAttachmentPermissionsBoundary>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a PermissionsBoundaryAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PermissionsBoundaryAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PermissionsBoundaryAttachment resources.
 */
export interface PermissionsBoundaryAttachmentState {
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    permissionSetArn?: pulumi.Input<string>;
    /**
     * The permissions boundary policy. See below.
     */
    permissionsBoundary?: pulumi.Input<inputs.ssoadmin.PermissionsBoundaryAttachmentPermissionsBoundary>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PermissionsBoundaryAttachment resource.
 */
export interface PermissionsBoundaryAttachmentArgs {
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    permissionSetArn: pulumi.Input<string>;
    /**
     * The permissions boundary policy. See below.
     */
    permissionsBoundary: pulumi.Input<inputs.ssoadmin.PermissionsBoundaryAttachmentPermissionsBoundary>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

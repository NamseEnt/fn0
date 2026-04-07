import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Global Accelerator Cross Account Attachment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.globalaccelerator.CrossAccountAttachment("example", {name: "example-cross-account-attachment"});
 * ```
 *
 * ### Usage with Optional Arguments
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.globalaccelerator.CrossAccountAttachment("example", {
 *     name: "example-cross-account-attachment",
 *     principals: ["123456789012"],
 *     resources: [{
 *         endpointId: "arn:aws:elasticloadbalancing:us-west-2:123456789012:loadbalancer/app/my-load-balancer/50dc6c495c0c9188",
 *         region: "us-west-2",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Global Accelerator cross-account attachment.
 *
 * Using `pulumi import`, import Global Accelerator Cross Account Attachment using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_globalaccelerator_cross_account_attachment.example arn:aws:globalaccelerator::012345678910:attachment/01234567-abcd-8910-efgh-123456789012
 */
export declare class CrossAccountAttachment extends pulumi.CustomResource {
    /**
     * Get an existing CrossAccountAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CrossAccountAttachmentState, opts?: pulumi.CustomResourceOptions): CrossAccountAttachment;
    /**
     * Returns true if the given object is an instance of CrossAccountAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CrossAccountAttachment;
    /**
     * ARN of the Cross Account Attachment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation Time when the Cross Account Attachment.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Last modified time of the Cross Account Attachment.
     */
    readonly lastModifiedTime: pulumi.Output<string>;
    /**
     * Name of the Cross Account Attachment.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * List of AWS account IDs that are allowed to associate resources with the accelerator.
     */
    readonly principals: pulumi.Output<string[] | undefined>;
    /**
     * List of resources to be associated with the accelerator.
     */
    readonly resources: pulumi.Output<outputs.globalaccelerator.CrossAccountAttachmentResource[] | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a CrossAccountAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: CrossAccountAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CrossAccountAttachment resources.
 */
export interface CrossAccountAttachmentState {
    /**
     * ARN of the Cross Account Attachment.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation Time when the Cross Account Attachment.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Last modified time of the Cross Account Attachment.
     */
    lastModifiedTime?: pulumi.Input<string>;
    /**
     * Name of the Cross Account Attachment.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * List of AWS account IDs that are allowed to associate resources with the accelerator.
     */
    principals?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of resources to be associated with the accelerator.
     */
    resources?: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CrossAccountAttachmentResource>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a CrossAccountAttachment resource.
 */
export interface CrossAccountAttachmentArgs {
    /**
     * Name of the Cross Account Attachment.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * List of AWS account IDs that are allowed to associate resources with the accelerator.
     */
    principals?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of resources to be associated with the accelerator.
     */
    resources?: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CrossAccountAttachmentResource>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

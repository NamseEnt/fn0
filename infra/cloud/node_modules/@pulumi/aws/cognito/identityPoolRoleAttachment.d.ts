import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Cognito Identity Pool Roles Attachment.
 *
 * ## Import
 *
 * Using `pulumi import`, import Cognito Identity Pool Roles Attachment using the Identity Pool ID. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/identityPoolRoleAttachment:IdentityPoolRoleAttachment example us-west-2:b64805ad-cb56-40ba-9ffc-f5d8207e6d42
 * ```
 */
export declare class IdentityPoolRoleAttachment extends pulumi.CustomResource {
    /**
     * Get an existing IdentityPoolRoleAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IdentityPoolRoleAttachmentState, opts?: pulumi.CustomResourceOptions): IdentityPoolRoleAttachment;
    /**
     * Returns true if the given object is an instance of IdentityPoolRoleAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IdentityPoolRoleAttachment;
    /**
     * An identity pool ID in the format `REGION_GUID`.
     */
    readonly identityPoolId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A List of Role Mapping.
     */
    readonly roleMappings: pulumi.Output<outputs.cognito.IdentityPoolRoleAttachmentRoleMapping[] | undefined>;
    /**
     * The map of roles associated with this pool. For a given role, the key will be either "authenticated" or "unauthenticated" and the value will be the Role ARN.
     */
    readonly roles: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a IdentityPoolRoleAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IdentityPoolRoleAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IdentityPoolRoleAttachment resources.
 */
export interface IdentityPoolRoleAttachmentState {
    /**
     * An identity pool ID in the format `REGION_GUID`.
     */
    identityPoolId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A List of Role Mapping.
     */
    roleMappings?: pulumi.Input<pulumi.Input<inputs.cognito.IdentityPoolRoleAttachmentRoleMapping>[]>;
    /**
     * The map of roles associated with this pool. For a given role, the key will be either "authenticated" or "unauthenticated" and the value will be the Role ARN.
     */
    roles?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a IdentityPoolRoleAttachment resource.
 */
export interface IdentityPoolRoleAttachmentArgs {
    /**
     * An identity pool ID in the format `REGION_GUID`.
     */
    identityPoolId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A List of Role Mapping.
     */
    roleMappings?: pulumi.Input<pulumi.Input<inputs.cognito.IdentityPoolRoleAttachmentRoleMapping>[]>;
    /**
     * The map of roles associated with this pool. For a given role, the key will be either "authenticated" or "unauthenticated" and the value will be the Role ARN.
     */
    roles: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

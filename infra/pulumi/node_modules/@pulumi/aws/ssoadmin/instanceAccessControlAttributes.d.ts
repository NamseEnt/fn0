import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Single Sign-On (SSO) ABAC Resource: https://docs.aws.amazon.com/singlesignon/latest/userguide/abac.html
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleInstanceAccessControlAttributes = new aws.ssoadmin.InstanceAccessControlAttributes("example", {
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     attributes: [
 *         {
 *             key: "name",
 *             values: [{
 *                 sources: ["${path:name.givenName}"],
 *             }],
 *         },
 *         {
 *             key: "last",
 *             values: [{
 *                 sources: ["${path:name.familyName}"],
 *             }],
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Account Assignments using the `instance_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/instanceAccessControlAttributes:InstanceAccessControlAttributes example arn:aws:sso:::instance/ssoins-0123456789abcdef
 * ```
 */
export declare class InstanceAccessControlAttributes extends pulumi.CustomResource {
    /**
     * Get an existing InstanceAccessControlAttributes resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceAccessControlAttributesState, opts?: pulumi.CustomResourceOptions): InstanceAccessControlAttributes;
    /**
     * Returns true if the given object is an instance of InstanceAccessControlAttributes.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceAccessControlAttributes;
    /**
     * See AccessControlAttribute for more details.
     */
    readonly attributes: pulumi.Output<outputs.ssoadmin.InstanceAccessControlAttributesAttribute[]>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly status: pulumi.Output<string>;
    readonly statusReason: pulumi.Output<string>;
    /**
     * Create a InstanceAccessControlAttributes resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceAccessControlAttributesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceAccessControlAttributes resources.
 */
export interface InstanceAccessControlAttributesState {
    /**
     * See AccessControlAttribute for more details.
     */
    attributes?: pulumi.Input<pulumi.Input<inputs.ssoadmin.InstanceAccessControlAttributesAttribute>[]>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
    statusReason?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InstanceAccessControlAttributes resource.
 */
export interface InstanceAccessControlAttributesArgs {
    /**
     * See AccessControlAttribute for more details.
     */
    attributes: pulumi.Input<pulumi.Input<inputs.ssoadmin.InstanceAccessControlAttributesAttribute>[]>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

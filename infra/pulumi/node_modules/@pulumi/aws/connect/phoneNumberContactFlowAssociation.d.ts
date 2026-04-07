import * as pulumi from "@pulumi/pulumi";
/**
 * Associates a flow with a phone number claimed to an Amazon Connect instance.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.PhoneNumberContactFlowAssociation("example", {
 *     phoneNumberId: exampleAwsConnectPhoneNumber.id,
 *     instanceId: exampleAwsConnectInstance.id,
 *     contactFlowId: exampleAwsConnectContactFlow.contactFlowId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_connect_phone_number_contact_flow_association` using the `phone_number_id`, `instance_id` and `contact_flow_id` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:connect/phoneNumberContactFlowAssociation:PhoneNumberContactFlowAssociation example 36727a4c-4683-4e49-880c-3347c61110a4,fa6c1691-e2eb-4487-bdb9-1aaed6268ebd,c4acdc79-395e-4280-a294-9062f56b07bb
 * ```
 */
export declare class PhoneNumberContactFlowAssociation extends pulumi.CustomResource {
    /**
     * Get an existing PhoneNumberContactFlowAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PhoneNumberContactFlowAssociationState, opts?: pulumi.CustomResourceOptions): PhoneNumberContactFlowAssociation;
    /**
     * Returns true if the given object is an instance of PhoneNumberContactFlowAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PhoneNumberContactFlowAssociation;
    /**
     * Contact flow ID.
     */
    readonly contactFlowId: pulumi.Output<string>;
    /**
     * Amazon Connect instance ID.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Phone number ID.
     */
    readonly phoneNumberId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a PhoneNumberContactFlowAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PhoneNumberContactFlowAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PhoneNumberContactFlowAssociation resources.
 */
export interface PhoneNumberContactFlowAssociationState {
    /**
     * Contact flow ID.
     */
    contactFlowId?: pulumi.Input<string>;
    /**
     * Amazon Connect instance ID.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Phone number ID.
     */
    phoneNumberId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PhoneNumberContactFlowAssociation resource.
 */
export interface PhoneNumberContactFlowAssociationArgs {
    /**
     * Contact flow ID.
     */
    contactFlowId: pulumi.Input<string>;
    /**
     * Amazon Connect instance ID.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Phone number ID.
     */
    phoneNumberId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

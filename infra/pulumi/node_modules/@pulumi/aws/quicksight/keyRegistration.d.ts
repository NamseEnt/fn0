import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Registers customer managed keys in a Amazon QuickSight account.
 *
 * > Deletion of this resource clears all CMK registrations from a QuickSight account. QuickSight then uses AWS owned keys to encrypt your resources.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.KeyRegistration("example", {keyRegistrations: [
 *     {
 *         keyArn: example1.arn,
 *     },
 *     {
 *         keyArn: example2.arn,
 *         defaultKey: true,
 *     },
 * ]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight key registration using the AWS account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/keyRegistration:KeyRegistration example "012345678901"
 * ```
 */
export declare class KeyRegistration extends pulumi.CustomResource {
    /**
     * Get an existing KeyRegistration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyRegistrationState, opts?: pulumi.CustomResourceOptions): KeyRegistration;
    /**
     * Returns true if the given object is an instance of KeyRegistration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyRegistration;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Registered keys. See key_registration.
     */
    readonly keyRegistrations: pulumi.Output<outputs.quicksight.KeyRegistrationKeyRegistration[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a KeyRegistration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: KeyRegistrationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyRegistration resources.
 */
export interface KeyRegistrationState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Registered keys. See key_registration.
     */
    keyRegistrations?: pulumi.Input<pulumi.Input<inputs.quicksight.KeyRegistrationKeyRegistration>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a KeyRegistration resource.
 */
export interface KeyRegistrationArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Registered keys. See key_registration.
     */
    keyRegistrations?: pulumi.Input<pulumi.Input<inputs.quicksight.KeyRegistrationKeyRegistration>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}

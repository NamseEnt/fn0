import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CloudFront Field-level Encryption Profile resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.cloudfront.PublicKey("example", {
 *     comment: "test public key",
 *     encodedKey: std.file({
 *         input: "public_key.pem",
 *     }).then(invoke => invoke.result),
 *     name: "test_key",
 * });
 * const test = new aws.cloudfront.FieldLevelEncryptionProfile("test", {
 *     comment: "test comment",
 *     name: "test profile",
 *     encryptionEntities: {
 *         items: [{
 *             publicKeyId: example.id,
 *             providerId: "test provider",
 *             fieldPatterns: {
 *                 items: ["DateOfBirth"],
 *             },
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cloudfront Field Level Encryption Profile using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/fieldLevelEncryptionProfile:FieldLevelEncryptionProfile profile K3D5EWEUDCCXON
 * ```
 */
export declare class FieldLevelEncryptionProfile extends pulumi.CustomResource {
    /**
     * Get an existing FieldLevelEncryptionProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FieldLevelEncryptionProfileState, opts?: pulumi.CustomResourceOptions): FieldLevelEncryptionProfile;
    /**
     * Returns true if the given object is an instance of FieldLevelEncryptionProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FieldLevelEncryptionProfile;
    /**
     * The Field Level Encryption Profile ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Internal value used by CloudFront to allow future updates to the Field Level Encryption Profile.
     */
    readonly callerReference: pulumi.Output<string>;
    /**
     * An optional comment about the Field Level Encryption Profile.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * The encryption entities config block for field-level encryption profiles that contains an attribute `items` which includes the encryption key and field pattern specifications.
     */
    readonly encryptionEntities: pulumi.Output<outputs.cloudfront.FieldLevelEncryptionProfileEncryptionEntities>;
    /**
     * The current version of the Field Level Encryption Profile. For example: `E2QWRUHAPOMQZL`.
     */
    readonly etag: pulumi.Output<string>;
    /**
     * The name of the Field Level Encryption Profile.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Create a FieldLevelEncryptionProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FieldLevelEncryptionProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FieldLevelEncryptionProfile resources.
 */
export interface FieldLevelEncryptionProfileState {
    /**
     * The Field Level Encryption Profile ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Internal value used by CloudFront to allow future updates to the Field Level Encryption Profile.
     */
    callerReference?: pulumi.Input<string>;
    /**
     * An optional comment about the Field Level Encryption Profile.
     */
    comment?: pulumi.Input<string>;
    /**
     * The encryption entities config block for field-level encryption profiles that contains an attribute `items` which includes the encryption key and field pattern specifications.
     */
    encryptionEntities?: pulumi.Input<inputs.cloudfront.FieldLevelEncryptionProfileEncryptionEntities>;
    /**
     * The current version of the Field Level Encryption Profile. For example: `E2QWRUHAPOMQZL`.
     */
    etag?: pulumi.Input<string>;
    /**
     * The name of the Field Level Encryption Profile.
     */
    name?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FieldLevelEncryptionProfile resource.
 */
export interface FieldLevelEncryptionProfileArgs {
    /**
     * An optional comment about the Field Level Encryption Profile.
     */
    comment?: pulumi.Input<string>;
    /**
     * The encryption entities config block for field-level encryption profiles that contains an attribute `items` which includes the encryption key and field pattern specifications.
     */
    encryptionEntities: pulumi.Input<inputs.cloudfront.FieldLevelEncryptionProfileEncryptionEntities>;
    /**
     * The name of the Field Level Encryption Profile.
     */
    name?: pulumi.Input<string>;
}

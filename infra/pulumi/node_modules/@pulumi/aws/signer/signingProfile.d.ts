import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a Signer Signing Profile. A signing profile contains information about the code signing configuration parameters that can be used by a given code signing user.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testSp = new aws.signer.SigningProfile("test_sp", {platformId: "AWSLambda-SHA384-ECDSA"});
 * const prodSp = new aws.signer.SigningProfile("prod_sp", {
 *     platformId: "AWSLambda-SHA384-ECDSA",
 *     namePrefix: "prod_sp_",
 *     signatureValidityPeriod: {
 *         value: 5,
 *         type: "YEARS",
 *     },
 *     tags: {
 *         tag1: "value1",
 *         tag2: "value2",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Signer signing profiles using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:signer/signingProfile:SigningProfile test_signer_signing_profile test_sp_DdW3Mk1foYL88fajut4mTVFGpuwfd4ACO6ANL0D1uIj7lrn8adK
 * ```
 */
export declare class SigningProfile extends pulumi.CustomResource {
    /**
     * Get an existing SigningProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SigningProfileState, opts?: pulumi.CustomResourceOptions): SigningProfile;
    /**
     * Returns true if the given object is an instance of SigningProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SigningProfile;
    /**
     * The Amazon Resource Name (ARN) for the signing profile.
     */
    readonly arn: pulumi.Output<string>;
    readonly name: pulumi.Output<string>;
    readonly namePrefix: pulumi.Output<string>;
    /**
     * A human-readable name for the signing platform associated with the signing profile.
     */
    readonly platformDisplayName: pulumi.Output<string>;
    /**
     * The ID of the platform that is used by the target signing profile.
     */
    readonly platformId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Revocation information for a signing profile. See `revocationRecord` Block below for details.
     */
    readonly revocationRecords: pulumi.Output<outputs.signer.SigningProfileRevocationRecord[]>;
    /**
     * The validity period for a signing job. See `signatureValidityPeriod` Block below for details.
     */
    readonly signatureValidityPeriod: pulumi.Output<outputs.signer.SigningProfileSignatureValidityPeriod>;
    /**
     * The AWS Certificate Manager certificate that will be used to sign code with the new signing profile. See `signingMaterial` Block below for details.
     */
    readonly signingMaterial: pulumi.Output<outputs.signer.SigningProfileSigningMaterial>;
    /**
     * Map of key-value pairs for signing. These can include any information that you want to use during signing.
     */
    readonly signingParameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The status of the target signing profile.
     */
    readonly status: pulumi.Output<string>;
    /**
     * A list of tags associated with the signing profile. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The current version of the signing profile.
     */
    readonly version: pulumi.Output<string>;
    /**
     * The signing profile ARN, including the profile version.
     */
    readonly versionArn: pulumi.Output<string>;
    /**
     * Create a SigningProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SigningProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SigningProfile resources.
 */
export interface SigningProfileState {
    /**
     * The Amazon Resource Name (ARN) for the signing profile.
     */
    arn?: pulumi.Input<string>;
    name?: pulumi.Input<string>;
    namePrefix?: pulumi.Input<string>;
    /**
     * A human-readable name for the signing platform associated with the signing profile.
     */
    platformDisplayName?: pulumi.Input<string>;
    /**
     * The ID of the platform that is used by the target signing profile.
     */
    platformId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Revocation information for a signing profile. See `revocationRecord` Block below for details.
     */
    revocationRecords?: pulumi.Input<pulumi.Input<inputs.signer.SigningProfileRevocationRecord>[]>;
    /**
     * The validity period for a signing job. See `signatureValidityPeriod` Block below for details.
     */
    signatureValidityPeriod?: pulumi.Input<inputs.signer.SigningProfileSignatureValidityPeriod>;
    /**
     * The AWS Certificate Manager certificate that will be used to sign code with the new signing profile. See `signingMaterial` Block below for details.
     */
    signingMaterial?: pulumi.Input<inputs.signer.SigningProfileSigningMaterial>;
    /**
     * Map of key-value pairs for signing. These can include any information that you want to use during signing.
     */
    signingParameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The status of the target signing profile.
     */
    status?: pulumi.Input<string>;
    /**
     * A list of tags associated with the signing profile. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The current version of the signing profile.
     */
    version?: pulumi.Input<string>;
    /**
     * The signing profile ARN, including the profile version.
     */
    versionArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SigningProfile resource.
 */
export interface SigningProfileArgs {
    name?: pulumi.Input<string>;
    namePrefix?: pulumi.Input<string>;
    /**
     * The ID of the platform that is used by the target signing profile.
     */
    platformId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The validity period for a signing job. See `signatureValidityPeriod` Block below for details.
     */
    signatureValidityPeriod?: pulumi.Input<inputs.signer.SigningProfileSignatureValidityPeriod>;
    /**
     * The AWS Certificate Manager certificate that will be used to sign code with the new signing profile. See `signingMaterial` Block below for details.
     */
    signingMaterial?: pulumi.Input<inputs.signer.SigningProfileSigningMaterial>;
    /**
     * Map of key-value pairs for signing. These can include any information that you want to use during signing.
     */
    signingParameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A list of tags associated with the signing profile. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

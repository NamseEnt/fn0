import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail Key Pair for use with Lightsail Instances. Use this resource to create or import key pairs that are separate from EC2 Key Pairs and required for Lightsail instances.
 *
 * > **Note:** Lightsail is currently only supported in a limited number of AWS Regions, please see ["Regions and Availability Zones in Amazon Lightsail"](https://lightsail.aws.amazon.com/ls/docs/overview/article/understanding-regions-and-availability-zones-in-amazon-lightsail) for more details
 *
 * ## Example Usage
 *
 * ### Create New Key Pair
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.KeyPair("example", {name: "example"});
 * ```
 *
 * ### Create New Key Pair with PGP Encrypted Private Key
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.KeyPair("example", {
 *     name: "example",
 *     pgpKey: "keybase:keybaseusername",
 * });
 * ```
 *
 * ### Existing Public Key Import
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.lightsail.KeyPair("example", {
 *     name: "example",
 *     publicKey: std.file({
 *         input: "~/.ssh/id_rsa.pub",
 *     }).then(invoke => invoke.result),
 * });
 * ```
 *
 * ## Import
 *
 * You cannot import Lightsail Key Pairs because the private and public key are only available on initial creation.
 */
export declare class KeyPair extends pulumi.CustomResource {
    /**
     * Get an existing KeyPair resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyPairState, opts?: pulumi.CustomResourceOptions): KeyPair;
    /**
     * Returns true if the given object is an instance of KeyPair.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyPair;
    /**
     * ARN of the Lightsail key pair.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * MD5 public key fingerprint for the encrypted private key.
     */
    readonly encryptedFingerprint: pulumi.Output<string>;
    /**
     * Private key material, base 64 encoded and encrypted with the given `pgpKey`. This is only populated when creating a new key and `pgpKey` is supplied.
     */
    readonly encryptedPrivateKey: pulumi.Output<string>;
    /**
     * MD5 public key fingerprint as specified in section 4 of RFC 4716.
     */
    readonly fingerprint: pulumi.Output<string>;
    /**
     * Name of the Lightsail Key Pair. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * PGP key to encrypt the resulting private key material. Only used when creating a new key pair.
     */
    readonly pgpKey: pulumi.Output<string | undefined>;
    /**
     * Private key, base64 encoded. This is only populated when creating a new key, and when no `pgpKey` is provided.
     */
    readonly privateKey: pulumi.Output<string>;
    /**
     * Public key material. This public key will be imported into Lightsail.
     */
    readonly publicKey: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **Note:** A PGP key is not required, however it is strongly encouraged. Without a PGP key, the private key material will be stored in state unencrypted. `pgpKey` is ignored if `publicKey` is supplied.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a KeyPair resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: KeyPairArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyPair resources.
 */
export interface KeyPairState {
    /**
     * ARN of the Lightsail key pair.
     */
    arn?: pulumi.Input<string>;
    /**
     * MD5 public key fingerprint for the encrypted private key.
     */
    encryptedFingerprint?: pulumi.Input<string>;
    /**
     * Private key material, base 64 encoded and encrypted with the given `pgpKey`. This is only populated when creating a new key and `pgpKey` is supplied.
     */
    encryptedPrivateKey?: pulumi.Input<string>;
    /**
     * MD5 public key fingerprint as specified in section 4 of RFC 4716.
     */
    fingerprint?: pulumi.Input<string>;
    /**
     * Name of the Lightsail Key Pair. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * PGP key to encrypt the resulting private key material. Only used when creating a new key pair.
     */
    pgpKey?: pulumi.Input<string>;
    /**
     * Private key, base64 encoded. This is only populated when creating a new key, and when no `pgpKey` is provided.
     */
    privateKey?: pulumi.Input<string>;
    /**
     * Public key material. This public key will be imported into Lightsail.
     */
    publicKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **Note:** A PGP key is not required, however it is strongly encouraged. Without a PGP key, the private key material will be stored in state unencrypted. `pgpKey` is ignored if `publicKey` is supplied.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a KeyPair resource.
 */
export interface KeyPairArgs {
    /**
     * Name of the Lightsail Key Pair. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * PGP key to encrypt the resulting private key material. Only used when creating a new key pair.
     */
    pgpKey?: pulumi.Input<string>;
    /**
     * Public key material. This public key will be imported into Lightsail.
     */
    publicKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **Note:** A PGP key is not required, however it is strongly encouraged. Without a PGP key, the private key material will be stored in state unencrypted. `pgpKey` is ignored if `publicKey` is supplied.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Route 53 Key Signing Key. To manage Domain Name System Security Extensions (DNSSEC) for a Hosted Zone, see the `aws.route53.HostedZoneDnsSec` resource. For more information about managing DNSSEC in Route 53, see the [Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-configuring-dnssec.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.kms.Key("example", {
 *     customerMasterKeySpec: "ECC_NIST_P256",
 *     deletionWindowInDays: 7,
 *     keyUsage: "SIGN_VERIFY",
 *     policy: JSON.stringify({
 *         Statement: [
 *             {
 *                 Action: [
 *                     "kms:DescribeKey",
 *                     "kms:GetPublicKey",
 *                     "kms:Sign",
 *                 ],
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "dnssec-route53.amazonaws.com",
 *                 },
 *                 Sid: "Allow Route 53 DNSSEC Service",
 *                 Resource: "*",
 *                 Condition: {
 *                     StringEquals: {
 *                         "aws:SourceAccount": current.then(current => current.accountId),
 *                     },
 *                     ArnLike: {
 *                         "aws:SourceArn": "arn:aws:route53:::hostedzone/*",
 *                     },
 *                 },
 *             },
 *             {
 *                 Action: "kms:CreateGrant",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "dnssec-route53.amazonaws.com",
 *                 },
 *                 Sid: "Allow Route 53 DNSSEC Service to CreateGrant",
 *                 Resource: "*",
 *                 Condition: {
 *                     Bool: {
 *                         "kms:GrantIsForAWSResource": "true",
 *                     },
 *                 },
 *             },
 *             {
 *                 Action: "kms:*",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     AWS: current.then(current => `arn:aws:iam::${current.accountId}:root`),
 *                 },
 *                 Resource: "*",
 *                 Sid: "Enable IAM User Permissions",
 *             },
 *         ],
 *         Version: "2012-10-17",
 *     }),
 * });
 * const exampleZone = new aws.route53.Zone("example", {name: "example.com"});
 * const exampleKeySigningKey = new aws.route53.KeySigningKey("example", {
 *     hostedZoneId: test.id,
 *     keyManagementServiceArn: testAwsKmsKey.arn,
 *     name: "example",
 * });
 * const exampleHostedZoneDnsSec = new aws.route53.HostedZoneDnsSec("example", {hostedZoneId: exampleKeySigningKey.hostedZoneId}, {
 *     dependsOn: [exampleKeySigningKey],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_route53_key_signing_key` resources using the Route 53 Hosted Zone identifier and KMS Key identifier, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:route53/keySigningKey:KeySigningKey example Z1D633PJN98FT9,example
 * ```
 */
export declare class KeySigningKey extends pulumi.CustomResource {
    /**
     * Get an existing KeySigningKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeySigningKeyState, opts?: pulumi.CustomResourceOptions): KeySigningKey;
    /**
     * Returns true if the given object is an instance of KeySigningKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeySigningKey;
    /**
     * A string used to represent the delegation signer digest algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.3](https://tools.ietf.org/html/rfc8624#section-3.3).
     */
    readonly digestAlgorithmMnemonic: pulumi.Output<string>;
    /**
     * An integer used to represent the delegation signer digest algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.3](https://tools.ietf.org/html/rfc8624#section-3.3).
     */
    readonly digestAlgorithmType: pulumi.Output<number>;
    /**
     * A cryptographic digest of a DNSKEY resource record (RR). DNSKEY records are used to publish the public key that resolvers can use to verify DNSSEC signatures that are used to secure certain kinds of information provided by the DNS system.
     */
    readonly digestValue: pulumi.Output<string>;
    /**
     * A string that represents a DNSKEY record.
     */
    readonly dnskeyRecord: pulumi.Output<string>;
    /**
     * A string that represents a delegation signer (DS) record.
     */
    readonly dsRecord: pulumi.Output<string>;
    /**
     * An integer that specifies how the key is used. For key-signing key (KSK), this value is always 257.
     */
    readonly flag: pulumi.Output<number>;
    /**
     * Identifier of the Route 53 Hosted Zone.
     */
    readonly hostedZoneId: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Management Service (KMS) Key. This must be unique for each key-signing key (KSK) in a single hosted zone. This key must be in the `us-east-1` Region and meet certain requirements, which are described in the [Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-configuring-dnssec-cmk-requirements.html) and [Route 53 API Reference](https://docs.aws.amazon.com/Route53/latest/APIReference/API_CreateKeySigningKey.html).
     */
    readonly keyManagementServiceArn: pulumi.Output<string>;
    /**
     * An integer used to identify the DNSSEC record for the domain name. The process used to calculate the value is described in [RFC-4034 Appendix B](https://tools.ietf.org/rfc/rfc4034.txt).
     */
    readonly keyTag: pulumi.Output<number>;
    /**
     * Name of the key-signing key (KSK). Must be unique for each key-signing key in the same hosted zone.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * The public key, represented as a Base64 encoding, as required by [RFC-4034 Page 5](https://tools.ietf.org/rfc/rfc4034.txt).
     */
    readonly publicKey: pulumi.Output<string>;
    /**
     * A string used to represent the signing algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.1](https://tools.ietf.org/html/rfc8624#section-3.1).
     */
    readonly signingAlgorithmMnemonic: pulumi.Output<string>;
    /**
     * An integer used to represent the signing algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.1](https://tools.ietf.org/html/rfc8624#section-3.1).
     */
    readonly signingAlgorithmType: pulumi.Output<number>;
    /**
     * Status of the key-signing key (KSK). Valid values: `ACTIVE`, `INACTIVE`. Defaults to `ACTIVE`.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Create a KeySigningKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KeySigningKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeySigningKey resources.
 */
export interface KeySigningKeyState {
    /**
     * A string used to represent the delegation signer digest algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.3](https://tools.ietf.org/html/rfc8624#section-3.3).
     */
    digestAlgorithmMnemonic?: pulumi.Input<string>;
    /**
     * An integer used to represent the delegation signer digest algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.3](https://tools.ietf.org/html/rfc8624#section-3.3).
     */
    digestAlgorithmType?: pulumi.Input<number>;
    /**
     * A cryptographic digest of a DNSKEY resource record (RR). DNSKEY records are used to publish the public key that resolvers can use to verify DNSSEC signatures that are used to secure certain kinds of information provided by the DNS system.
     */
    digestValue?: pulumi.Input<string>;
    /**
     * A string that represents a DNSKEY record.
     */
    dnskeyRecord?: pulumi.Input<string>;
    /**
     * A string that represents a delegation signer (DS) record.
     */
    dsRecord?: pulumi.Input<string>;
    /**
     * An integer that specifies how the key is used. For key-signing key (KSK), this value is always 257.
     */
    flag?: pulumi.Input<number>;
    /**
     * Identifier of the Route 53 Hosted Zone.
     */
    hostedZoneId?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Management Service (KMS) Key. This must be unique for each key-signing key (KSK) in a single hosted zone. This key must be in the `us-east-1` Region and meet certain requirements, which are described in the [Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-configuring-dnssec-cmk-requirements.html) and [Route 53 API Reference](https://docs.aws.amazon.com/Route53/latest/APIReference/API_CreateKeySigningKey.html).
     */
    keyManagementServiceArn?: pulumi.Input<string>;
    /**
     * An integer used to identify the DNSSEC record for the domain name. The process used to calculate the value is described in [RFC-4034 Appendix B](https://tools.ietf.org/rfc/rfc4034.txt).
     */
    keyTag?: pulumi.Input<number>;
    /**
     * Name of the key-signing key (KSK). Must be unique for each key-signing key in the same hosted zone.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The public key, represented as a Base64 encoding, as required by [RFC-4034 Page 5](https://tools.ietf.org/rfc/rfc4034.txt).
     */
    publicKey?: pulumi.Input<string>;
    /**
     * A string used to represent the signing algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.1](https://tools.ietf.org/html/rfc8624#section-3.1).
     */
    signingAlgorithmMnemonic?: pulumi.Input<string>;
    /**
     * An integer used to represent the signing algorithm. This value must follow the guidelines provided by [RFC-8624 Section 3.1](https://tools.ietf.org/html/rfc8624#section-3.1).
     */
    signingAlgorithmType?: pulumi.Input<number>;
    /**
     * Status of the key-signing key (KSK). Valid values: `ACTIVE`, `INACTIVE`. Defaults to `ACTIVE`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a KeySigningKey resource.
 */
export interface KeySigningKeyArgs {
    /**
     * Identifier of the Route 53 Hosted Zone.
     */
    hostedZoneId: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Management Service (KMS) Key. This must be unique for each key-signing key (KSK) in a single hosted zone. This key must be in the `us-east-1` Region and meet certain requirements, which are described in the [Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-configuring-dnssec-cmk-requirements.html) and [Route 53 API Reference](https://docs.aws.amazon.com/Route53/latest/APIReference/API_CreateKeySigningKey.html).
     */
    keyManagementServiceArn: pulumi.Input<string>;
    /**
     * Name of the key-signing key (KSK). Must be unique for each key-signing key in the same hosted zone.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Status of the key-signing key (KSK). Valid values: `ACTIVE`, `INACTIVE`. Defaults to `ACTIVE`.
     */
    status?: pulumi.Input<string>;
}

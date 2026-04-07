import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage a [delegation signer record](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-configuring-dnssec-enable-signing.html#dns-configuring-dnssec-enable-signing-step-1) in the parent DNS zone for domains registered with Route53.
 *
 * ## Example Usage
 *
 * ### Basic Usage
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
 * const exampleDelegationSignerRecord = new aws.route53domains.DelegationSignerRecord("example", {
 *     domainName: "example.com",
 *     signingAttributes: {
 *         algorithm: exampleKeySigningKey.signingAlgorithmType,
 *         flags: exampleKeySigningKey.flag,
 *         publicKey: exampleKeySigningKey.publicKey,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import delegation signer records using the domain name and DNSSEC key ID, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:route53domains/delegationSignerRecord:DelegationSignerRecord example example.com,40DE3534F5324DBDAC598ACEDB5B1E26A5368732D9C791D1347E4FBDDF6FC343
 * ```
 */
export declare class DelegationSignerRecord extends pulumi.CustomResource {
    /**
     * Get an existing DelegationSignerRecord resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DelegationSignerRecordState, opts?: pulumi.CustomResourceOptions): DelegationSignerRecord;
    /**
     * Returns true if the given object is an instance of DelegationSignerRecord.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DelegationSignerRecord;
    /**
     * An ID assigned to the created DS record.
     */
    readonly dnssecKeyId: pulumi.Output<string>;
    /**
     * The name of the domain that will have its parent DNS zone updated with the Delegation Signer record.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The information about a key, including the algorithm, public key-value, and flags.
     */
    readonly signingAttributes: pulumi.Output<outputs.route53domains.DelegationSignerRecordSigningAttributes | undefined>;
    readonly timeouts: pulumi.Output<outputs.route53domains.DelegationSignerRecordTimeouts | undefined>;
    /**
     * Create a DelegationSignerRecord resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DelegationSignerRecordArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DelegationSignerRecord resources.
 */
export interface DelegationSignerRecordState {
    /**
     * An ID assigned to the created DS record.
     */
    dnssecKeyId?: pulumi.Input<string>;
    /**
     * The name of the domain that will have its parent DNS zone updated with the Delegation Signer record.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The information about a key, including the algorithm, public key-value, and flags.
     */
    signingAttributes?: pulumi.Input<inputs.route53domains.DelegationSignerRecordSigningAttributes>;
    timeouts?: pulumi.Input<inputs.route53domains.DelegationSignerRecordTimeouts>;
}
/**
 * The set of arguments for constructing a DelegationSignerRecord resource.
 */
export interface DelegationSignerRecordArgs {
    /**
     * The name of the domain that will have its parent DNS zone updated with the Delegation Signer record.
     */
    domainName: pulumi.Input<string>;
    /**
     * The information about a key, including the algorithm, public key-value, and flags.
     */
    signingAttributes?: pulumi.Input<inputs.route53domains.DelegationSignerRecordSigningAttributes>;
    timeouts?: pulumi.Input<inputs.route53domains.DelegationSignerRecordTimeouts>;
}

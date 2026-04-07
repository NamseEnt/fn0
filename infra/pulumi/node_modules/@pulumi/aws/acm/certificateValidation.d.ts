import * as pulumi from "@pulumi/pulumi";
/**
 * This resource represents a successful validation of an ACM certificate in concert
 * with other resources.
 *
 * Most commonly, this resource is used together with `aws.route53.Record` and
 * `aws.acm.Certificate` to request a DNS validated certificate,
 * deploy the required validation records and wait for validation to complete.
 *
 * > **WARNING:** This resource implements a part of the validation workflow. It does not represent a real-world entity in AWS, therefore changing or deleting this resource on its own has no immediate effect.
 *
 * ## Example Usage
 *
 * ### DNS Validation with Route 53
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleCertificate = new aws.acm.Certificate("example", {
 *     domainName: "example.com",
 *     validationMethod: "DNS",
 * });
 * const example = aws.route53.getZone({
 *     name: "example.com",
 *     privateZone: false,
 * });
 * const exampleRecord: aws.route53.Record[] = [];
 * exampleCertificate.domainValidationOptions.apply(domainValidationOptions => {
 *     for (const range of Object.entries(domainValidationOptions.reduce((__obj, dvo) => ({ ...__obj, [dvo.domainName]: {
 *         name: dvo.resourceRecordName,
 *         record: dvo.resourceRecordValue,
 *         type: dvo.resourceRecordType,
 *     } }))).map(([k, v]) => ({key: k, value: v}))) {
 *         exampleRecord.push(new aws.route53.Record(`example-${range.key}`, {
 *             allowOverwrite: true,
 *             name: range.value.name,
 *             records: [range.value.record],
 *             ttl: 60,
 *             type: aws.route53.RecordType[range.value.type],
 *             zoneId: example.then(example => example.zoneId),
 *         }));
 *     }
 * });
 * const exampleCertificateValidation = new aws.acm.CertificateValidation("example", {
 *     certificateArn: exampleCertificate.arn,
 *     validationRecordFqdns: exampleRecord.apply(exampleRecord => exampleRecord.map(record => (record.fqdn))),
 * });
 * const exampleListener = new aws.lb.Listener("example", {certificateArn: exampleCertificateValidation.certificateArn});
 * ```
 *
 * ### Alternative Domains DNS Validation with Route 53
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.acm.Certificate("example", {
 *     domainName: "example.com",
 *     subjectAlternativeNames: [
 *         "www.example.com",
 *         "example.org",
 *     ],
 *     validationMethod: "DNS",
 * });
 * const exampleCom = aws.route53.getZone({
 *     name: "example.com",
 *     privateZone: false,
 * });
 * const exampleOrg = aws.route53.getZone({
 *     name: "example.org",
 *     privateZone: false,
 * });
 * const exampleRecord: aws.route53.Record[] = [];
 * pulumi.all([example.domainValidationOptions, dvo.domainName == "example.org" ? exampleOrg.then(exampleOrg => exampleOrg.zoneId) : exampleCom.then(exampleCom => exampleCom.zoneId)]).apply(([domainValidationOptions, value]) => {
 *     for (const range of Object.entries(domainValidationOptions.reduce((__obj, dvo) => ({ ...__obj, [dvo.domainName]: {
 *         name: dvo.resourceRecordName,
 *         record: dvo.resourceRecordValue,
 *         type: dvo.resourceRecordType,
 *         zoneId: value,
 *     } }))).map(([k, v]) => ({key: k, value: v}))) {
 *         exampleRecord.push(new aws.route53.Record(`example-${range.key}`, {
 *             allowOverwrite: true,
 *             name: range.value.name,
 *             records: [range.value.record],
 *             ttl: 60,
 *             type: aws.route53.RecordType[range.value.type],
 *             zoneId: range.value.zoneId,
 *         }));
 *     }
 * });
 * const exampleCertificateValidation = new aws.acm.CertificateValidation("example", {
 *     certificateArn: example.arn,
 *     validationRecordFqdns: exampleRecord.apply(exampleRecord => exampleRecord.map(record => (record.fqdn))),
 * });
 * const exampleListener = new aws.lb.Listener("example", {certificateArn: exampleCertificateValidation.certificateArn});
 * ```
 *
 * ### Email Validation
 *
 * In this situation, the resource is simply a waiter for manual email approval of ACM certificates.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.acm.Certificate("example", {
 *     domainName: "example.com",
 *     validationMethod: "EMAIL",
 * });
 * const exampleCertificateValidation = new aws.acm.CertificateValidation("example", {certificateArn: example.arn});
 * ```
 */
export declare class CertificateValidation extends pulumi.CustomResource {
    /**
     * Get an existing CertificateValidation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CertificateValidationState, opts?: pulumi.CustomResourceOptions): CertificateValidation;
    /**
     * Returns true if the given object is an instance of CertificateValidation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CertificateValidation;
    /**
     * ARN of the certificate that is being validated.
     */
    readonly certificateArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of FQDNs that implement the validation. Only valid for DNS validation method ACM certificates. If this is set, the resource can implement additional sanity checks and has an explicit dependency on the resource that is implementing the validation
     */
    readonly validationRecordFqdns: pulumi.Output<string[] | undefined>;
    /**
     * Create a CertificateValidation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CertificateValidationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CertificateValidation resources.
 */
export interface CertificateValidationState {
    /**
     * ARN of the certificate that is being validated.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of FQDNs that implement the validation. Only valid for DNS validation method ACM certificates. If this is set, the resource can implement additional sanity checks and has an explicit dependency on the resource that is implementing the validation
     */
    validationRecordFqdns?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a CertificateValidation resource.
 */
export interface CertificateValidationArgs {
    /**
     * ARN of the certificate that is being validated.
     */
    certificateArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of FQDNs that implement the validation. Only valid for DNS validation method ACM certificates. If this is set, the resource can implement additional sanity checks and has an explicit dependency on the resource that is implementing the validation
     */
    validationRecordFqdns?: pulumi.Input<pulumi.Input<string>[]>;
}

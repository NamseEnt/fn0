import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Assigns a static reverse DNS record to an Elastic IP addresses. See [Using reverse DNS for email applications](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/elastic-ip-addresses-eip.html#Using_Elastic_Addressing_Reverse_DNS).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Eip("example", {domain: "vpc"});
 * const exampleRecord = new aws.route53.Record("example", {
 *     zoneId: main.zoneId,
 *     name: "reverse",
 *     type: aws.route53.RecordType.A,
 *     records: [example.publicIp],
 * });
 * const exampleEipDomainName = new aws.ec2.EipDomainName("example", {
 *     allocationId: example.allocationId,
 *     domainName: exampleRecord.fqdn,
 * });
 * ```
 */
export declare class EipDomainName extends pulumi.CustomResource {
    /**
     * Get an existing EipDomainName resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EipDomainNameState, opts?: pulumi.CustomResourceOptions): EipDomainName;
    /**
     * Returns true if the given object is an instance of EipDomainName.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EipDomainName;
    /**
     * The allocation ID.
     */
    readonly allocationId: pulumi.Output<string>;
    /**
     * The domain name to modify for the IP address.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The DNS pointer (PTR) record for the IP address.
     */
    readonly ptrRecord: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ec2.EipDomainNameTimeouts | undefined>;
    /**
     * Create a EipDomainName resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EipDomainNameArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EipDomainName resources.
 */
export interface EipDomainNameState {
    /**
     * The allocation ID.
     */
    allocationId?: pulumi.Input<string>;
    /**
     * The domain name to modify for the IP address.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The DNS pointer (PTR) record for the IP address.
     */
    ptrRecord?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.EipDomainNameTimeouts>;
}
/**
 * The set of arguments for constructing a EipDomainName resource.
 */
export interface EipDomainNameArgs {
    /**
     * The allocation ID.
     */
    allocationId: pulumi.Input<string>;
    /**
     * The domain name to modify for the IP address.
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.EipDomainNameTimeouts>;
}

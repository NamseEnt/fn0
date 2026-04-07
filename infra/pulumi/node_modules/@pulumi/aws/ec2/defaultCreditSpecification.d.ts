import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS EC2 (Elastic Compute Cloud) Default Credit Specification.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.DefaultCreditSpecification("example", {
 *     instanceFamily: "t2",
 *     cpuCredits: "standard",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EC2 (Elastic Compute Cloud) Default Credit Specification using the `instance_family`. For example:
 *
 * console
 *
 * % pulumi import aws_ec2_default_credit_specification.example t2
 */
export declare class DefaultCreditSpecification extends pulumi.CustomResource {
    /**
     * Get an existing DefaultCreditSpecification resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DefaultCreditSpecificationState, opts?: pulumi.CustomResourceOptions): DefaultCreditSpecification;
    /**
     * Returns true if the given object is an instance of DefaultCreditSpecification.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DefaultCreditSpecification;
    /**
     * Credit option for CPU usage of the instance family. Valid values: `standard`, `unlimited`.
     */
    readonly cpuCredits: pulumi.Output<string>;
    /**
     * Instance family. Valid values are `t2`, `t3`, `t3a`, `t4g`.
     */
    readonly instanceFamily: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ec2.DefaultCreditSpecificationTimeouts | undefined>;
    /**
     * Create a DefaultCreditSpecification resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DefaultCreditSpecificationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DefaultCreditSpecification resources.
 */
export interface DefaultCreditSpecificationState {
    /**
     * Credit option for CPU usage of the instance family. Valid values: `standard`, `unlimited`.
     */
    cpuCredits?: pulumi.Input<string>;
    /**
     * Instance family. Valid values are `t2`, `t3`, `t3a`, `t4g`.
     */
    instanceFamily?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.DefaultCreditSpecificationTimeouts>;
}
/**
 * The set of arguments for constructing a DefaultCreditSpecification resource.
 */
export interface DefaultCreditSpecificationArgs {
    /**
     * Credit option for CPU usage of the instance family. Valid values: `standard`, `unlimited`.
     */
    cpuCredits: pulumi.Input<string>;
    /**
     * Instance family. Valid values are `t2`, `t3`, `t3a`, `t4g`.
     */
    instanceFamily: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.DefaultCreditSpecificationTimeouts>;
}

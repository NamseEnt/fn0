import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS IoT Billing Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iot.BillingGroup("example", {
 *     name: "example",
 *     properties: {
 *         description: "This is my billing group",
 *     },
 *     tags: {
 *         terraform: "true",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IoT Billing Groups using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/billingGroup:BillingGroup example example
 * ```
 */
export declare class BillingGroup extends pulumi.CustomResource {
    /**
     * Get an existing BillingGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BillingGroupState, opts?: pulumi.CustomResourceOptions): BillingGroup;
    /**
     * Returns true if the given object is an instance of BillingGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BillingGroup;
    /**
     * The ARN of the Billing Group.
     */
    readonly arn: pulumi.Output<string>;
    readonly metadatas: pulumi.Output<outputs.iot.BillingGroupMetadata[]>;
    /**
     * The name of the Billing Group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The Billing Group properties. Defined below.
     */
    readonly properties: pulumi.Output<outputs.iot.BillingGroupProperties | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The current version of the Billing Group record in the registry.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a BillingGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: BillingGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BillingGroup resources.
 */
export interface BillingGroupState {
    /**
     * The ARN of the Billing Group.
     */
    arn?: pulumi.Input<string>;
    metadatas?: pulumi.Input<pulumi.Input<inputs.iot.BillingGroupMetadata>[]>;
    /**
     * The name of the Billing Group.
     */
    name?: pulumi.Input<string>;
    /**
     * The Billing Group properties. Defined below.
     */
    properties?: pulumi.Input<inputs.iot.BillingGroupProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The current version of the Billing Group record in the registry.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a BillingGroup resource.
 */
export interface BillingGroupArgs {
    /**
     * The name of the Billing Group.
     */
    name?: pulumi.Input<string>;
    /**
     * The Billing Group properties. Defined below.
     */
    properties?: pulumi.Input<inputs.iot.BillingGroupProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}

import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Invoice Unit for organizational billing.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.invoicing.InvoiceUnit("example", {
 *     name: "example-unit",
 *     description: "Example invoice unit",
 *     invoiceReceiver: "123456789012",
 *     rules: [{
 *         linkedAccounts: ["098765432109"],
 *     }],
 *     tags: {
 *         Environment: "production",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Invoice Units using the ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:invoicing/invoiceUnit:InvoiceUnit example arn:aws:invoicing::123456789012:invoice-unit/example-id
 * ```
 */
export declare class InvoiceUnit extends pulumi.CustomResource {
    /**
     * Get an existing InvoiceUnit resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InvoiceUnitState, opts?: pulumi.CustomResourceOptions): InvoiceUnit;
    /**
     * Returns true if the given object is an instance of InvoiceUnit.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InvoiceUnit;
    /**
     * ARN of the invoice unit.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the invoice unit.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * AWS account ID that receives invoices for this unit. Cannot be changed after creation.
     */
    readonly invoiceReceiver: pulumi.Output<string>;
    /**
     * Timestamp when the invoice unit was last modified.
     */
    readonly lastModified: pulumi.Output<string>;
    /**
     * Unique name of the invoice unit. Cannot be changed after creation.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for invoice unit rules. See below.
     *
     * The following arguments are optional:
     */
    readonly rules: pulumi.Output<outputs.invoicing.InvoiceUnitRule[] | undefined>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Whether tax inheritance is disabled for this invoice unit.
     */
    readonly taxInheritanceDisabled: pulumi.Output<boolean>;
    readonly timeouts: pulumi.Output<outputs.invoicing.InvoiceUnitTimeouts | undefined>;
    /**
     * Create a InvoiceUnit resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InvoiceUnitArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InvoiceUnit resources.
 */
export interface InvoiceUnitState {
    /**
     * ARN of the invoice unit.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the invoice unit.
     */
    description?: pulumi.Input<string>;
    /**
     * AWS account ID that receives invoices for this unit. Cannot be changed after creation.
     */
    invoiceReceiver?: pulumi.Input<string>;
    /**
     * Timestamp when the invoice unit was last modified.
     */
    lastModified?: pulumi.Input<string>;
    /**
     * Unique name of the invoice unit. Cannot be changed after creation.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for invoice unit rules. See below.
     *
     * The following arguments are optional:
     */
    rules?: pulumi.Input<pulumi.Input<inputs.invoicing.InvoiceUnitRule>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Whether tax inheritance is disabled for this invoice unit.
     */
    taxInheritanceDisabled?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.invoicing.InvoiceUnitTimeouts>;
}
/**
 * The set of arguments for constructing a InvoiceUnit resource.
 */
export interface InvoiceUnitArgs {
    /**
     * Description of the invoice unit.
     */
    description?: pulumi.Input<string>;
    /**
     * AWS account ID that receives invoices for this unit. Cannot be changed after creation.
     */
    invoiceReceiver: pulumi.Input<string>;
    /**
     * Unique name of the invoice unit. Cannot be changed after creation.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for invoice unit rules. See below.
     *
     * The following arguments are optional:
     */
    rules?: pulumi.Input<pulumi.Input<inputs.invoicing.InvoiceUnitRule>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Whether tax inheritance is disabled for this invoice unit.
     */
    taxInheritanceDisabled?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.invoicing.InvoiceUnitTimeouts>;
}

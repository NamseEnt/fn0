import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Account VDM Attributes.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.AccountVdmAttributes("example", {
 *     vdmEnabled: "ENABLED",
 *     dashboardAttributes: {
 *         engagementMetrics: "ENABLED",
 *     },
 *     guardianAttributes: {
 *         optimizedSharedDelivery: "ENABLED",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Account VDM Attributes using the word `ses-account-vdm-attributes`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/accountVdmAttributes:AccountVdmAttributes example ses-account-vdm-attributes
 * ```
 */
export declare class AccountVdmAttributes extends pulumi.CustomResource {
    /**
     * Get an existing AccountVdmAttributes resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountVdmAttributesState, opts?: pulumi.CustomResourceOptions): AccountVdmAttributes;
    /**
     * Returns true if the given object is an instance of AccountVdmAttributes.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountVdmAttributes;
    /**
     * Specifies additional settings for your VDM configuration as applicable to the Dashboard.
     */
    readonly dashboardAttributes: pulumi.Output<outputs.sesv2.AccountVdmAttributesDashboardAttributes>;
    /**
     * Specifies additional settings for your VDM configuration as applicable to the Guardian.
     */
    readonly guardianAttributes: pulumi.Output<outputs.sesv2.AccountVdmAttributesGuardianAttributes>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the status of your VDM configuration. Valid values: `ENABLED`, `DISABLED`.
     *
     * The following arguments are optional:
     */
    readonly vdmEnabled: pulumi.Output<string>;
    /**
     * Create a AccountVdmAttributes resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccountVdmAttributesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountVdmAttributes resources.
 */
export interface AccountVdmAttributesState {
    /**
     * Specifies additional settings for your VDM configuration as applicable to the Dashboard.
     */
    dashboardAttributes?: pulumi.Input<inputs.sesv2.AccountVdmAttributesDashboardAttributes>;
    /**
     * Specifies additional settings for your VDM configuration as applicable to the Guardian.
     */
    guardianAttributes?: pulumi.Input<inputs.sesv2.AccountVdmAttributesGuardianAttributes>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the status of your VDM configuration. Valid values: `ENABLED`, `DISABLED`.
     *
     * The following arguments are optional:
     */
    vdmEnabled?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccountVdmAttributes resource.
 */
export interface AccountVdmAttributesArgs {
    /**
     * Specifies additional settings for your VDM configuration as applicable to the Dashboard.
     */
    dashboardAttributes?: pulumi.Input<inputs.sesv2.AccountVdmAttributesDashboardAttributes>;
    /**
     * Specifies additional settings for your VDM configuration as applicable to the Guardian.
     */
    guardianAttributes?: pulumi.Input<inputs.sesv2.AccountVdmAttributesGuardianAttributes>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the status of your VDM configuration. Valid values: `ENABLED`, `DISABLED`.
     *
     * The following arguments are optional:
     */
    vdmEnabled: pulumi.Input<string>;
}

import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Allows the application of pre-defined controls to organizational units. For more information on usage, please see the
 * [AWS Control Tower User Guide](https://docs.aws.amazon.com/controltower/latest/userguide/enable-guardrails.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const example = aws.organizations.getOrganization({});
 * const exampleGetOrganizationalUnits = example.then(example => aws.organizations.getOrganizationalUnits({
 *     parentId: example.roots?.[0]?.id,
 * }));
 * const exampleControlTowerControl = new aws.controltower.ControlTowerControl("example", {
 *     controlIdentifier: current.then(current => `arn:aws:controltower:${current.region}::control/AWS-GR_EC2_VOLUME_INUSE_CHECK`),
 *     targetIdentifier: exampleGetOrganizationalUnits.then(exampleGetOrganizationalUnits => .filter(x => x.name == "Infrastructure").map(x => (x.arn))[0]),
 *     parameters: [{
 *         key: "AllowedRegions",
 *         value: JSON.stringify(["us-east-1"]),
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Control Tower Controls using their `organizational_unit_arn/control_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:controltower/controlTowerControl:ControlTowerControl example arn:aws:organizations::123456789101:ou/o-qqaejywet/ou-qg5o-ufbhdtv3,arn:aws:controltower:us-east-1::control/WTDSMKDKDNLE
 * ```
 */
export declare class ControlTowerControl extends pulumi.CustomResource {
    /**
     * Get an existing ControlTowerControl resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ControlTowerControlState, opts?: pulumi.CustomResourceOptions): ControlTowerControl;
    /**
     * Returns true if the given object is an instance of ControlTowerControl.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ControlTowerControl;
    /**
     * The ARN of the EnabledControl resource.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the control. Only Strongly recommended and Elective controls are permitted, with the exception of the Region deny guardrail.
     */
    readonly controlIdentifier: pulumi.Output<string>;
    /**
     * Parameter values which are specified to configure the control when you enable it. See Parameters for more details.
     */
    readonly parameters: pulumi.Output<outputs.controltower.ControlTowerControlParameter[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the organizational unit.
     *
     * The following arguments are optional:
     */
    readonly targetIdentifier: pulumi.Output<string>;
    /**
     * Create a ControlTowerControl resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ControlTowerControlArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ControlTowerControl resources.
 */
export interface ControlTowerControlState {
    /**
     * The ARN of the EnabledControl resource.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the control. Only Strongly recommended and Elective controls are permitted, with the exception of the Region deny guardrail.
     */
    controlIdentifier?: pulumi.Input<string>;
    /**
     * Parameter values which are specified to configure the control when you enable it. See Parameters for more details.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.controltower.ControlTowerControlParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the organizational unit.
     *
     * The following arguments are optional:
     */
    targetIdentifier?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ControlTowerControl resource.
 */
export interface ControlTowerControlArgs {
    /**
     * The ARN of the control. Only Strongly recommended and Elective controls are permitted, with the exception of the Region deny guardrail.
     */
    controlIdentifier: pulumi.Input<string>;
    /**
     * Parameter values which are specified to configure the control when you enable it. See Parameters for more details.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.controltower.ControlTowerControlParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the organizational unit.
     *
     * The following arguments are optional:
     */
    targetIdentifier: pulumi.Input<string>;
}

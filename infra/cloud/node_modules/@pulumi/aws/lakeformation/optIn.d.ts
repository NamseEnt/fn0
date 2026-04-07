import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Lake Formation Opt In.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.OptIn("example", {});
 * ```
 */
export declare class OptIn extends pulumi.CustomResource {
    /**
     * Get an existing OptIn resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OptInState, opts?: pulumi.CustomResourceOptions): OptIn;
    /**
     * Returns true if the given object is an instance of OptIn.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OptIn;
    /**
     * Lake Formation condition, which applies to permissions and opt-ins that contain an expression.
     */
    readonly conditions: pulumi.Output<outputs.lakeformation.OptInCondition[] | undefined>;
    /**
     * Last modified date and time of the record.
     */
    readonly lastModified: pulumi.Output<string>;
    readonly lastUpdatedBy: pulumi.Output<string>;
    /**
     * Lake Formation principal. Supported principals are IAM users or IAM roles. See Principal for more details.
     */
    readonly principals: pulumi.Output<outputs.lakeformation.OptInPrincipal[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Structure for the resource. See Resource for more details.
     */
    readonly resourceDatas: pulumi.Output<outputs.lakeformation.OptInResourceData[] | undefined>;
    /**
     * Create a OptIn resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: OptInArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OptIn resources.
 */
export interface OptInState {
    /**
     * Lake Formation condition, which applies to permissions and opt-ins that contain an expression.
     */
    conditions?: pulumi.Input<pulumi.Input<inputs.lakeformation.OptInCondition>[]>;
    /**
     * Last modified date and time of the record.
     */
    lastModified?: pulumi.Input<string>;
    lastUpdatedBy?: pulumi.Input<string>;
    /**
     * Lake Formation principal. Supported principals are IAM users or IAM roles. See Principal for more details.
     */
    principals?: pulumi.Input<pulumi.Input<inputs.lakeformation.OptInPrincipal>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Structure for the resource. See Resource for more details.
     */
    resourceDatas?: pulumi.Input<pulumi.Input<inputs.lakeformation.OptInResourceData>[]>;
}
/**
 * The set of arguments for constructing a OptIn resource.
 */
export interface OptInArgs {
    /**
     * Lake Formation condition, which applies to permissions and opt-ins that contain an expression.
     */
    conditions?: pulumi.Input<pulumi.Input<inputs.lakeformation.OptInCondition>[]>;
    /**
     * Lake Formation principal. Supported principals are IAM users or IAM roles. See Principal for more details.
     */
    principals?: pulumi.Input<pulumi.Input<inputs.lakeformation.OptInPrincipal>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Structure for the resource. See Resource for more details.
     */
    resourceDatas?: pulumi.Input<pulumi.Input<inputs.lakeformation.OptInResourceData>[]>;
}

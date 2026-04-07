import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Import
 *
 * Using `pulumi import`, import Lex V2 Models Slot using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:lex/v2modelsSlot:V2modelsSlot example bot-1234,1,intent-5678,en-US,slot-9012
 * ```
 */
export declare class V2modelsSlot extends pulumi.CustomResource {
    /**
     * Get an existing V2modelsSlot resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: V2modelsSlotState, opts?: pulumi.CustomResourceOptions): V2modelsSlot;
    /**
     * Returns true if the given object is an instance of V2modelsSlot.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is V2modelsSlot;
    /**
     * Identifier of the bot associated with the slot.
     */
    readonly botId: pulumi.Output<string>;
    /**
     * Version of the bot associated with the slot.
     */
    readonly botVersion: pulumi.Output<string>;
    /**
     * Description of the slot.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Identifier of the intent that contains the slot.
     */
    readonly intentId: pulumi.Output<string>;
    /**
     * Identifier of the language and locale that the slot will be used in.
     */
    readonly localeId: pulumi.Output<string>;
    /**
     * Whether the slot returns multiple values in one response.
     * See the `multipleValuesSetting` argument reference below.
     */
    readonly multipleValuesSettings: pulumi.Output<outputs.lex.V2modelsSlotMultipleValuesSetting[] | undefined>;
    /**
     * Name of the slot.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Determines how slot values are used in Amazon CloudWatch logs.
     * See the `obfuscationSetting` argument reference below.
     */
    readonly obfuscationSettings: pulumi.Output<outputs.lex.V2modelsSlotObfuscationSetting[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Unique identifier associated with the slot.
     */
    readonly slotId: pulumi.Output<string>;
    /**
     * Unique identifier for the slot type associated with this slot.
     */
    readonly slotTypeId: pulumi.Output<string>;
    /**
     * Specifications for the constituent sub slots and the expression for the composite slot.
     * See the `subSlotSetting` argument reference below.
     */
    readonly subSlotSettings: pulumi.Output<outputs.lex.V2modelsSlotSubSlotSetting[] | undefined>;
    readonly timeouts: pulumi.Output<outputs.lex.V2modelsSlotTimeouts | undefined>;
    readonly valueElicitationSetting: pulumi.Output<outputs.lex.V2modelsSlotValueElicitationSetting | undefined>;
    /**
     * Create a V2modelsSlot resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: V2modelsSlotArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering V2modelsSlot resources.
 */
export interface V2modelsSlotState {
    /**
     * Identifier of the bot associated with the slot.
     */
    botId?: pulumi.Input<string>;
    /**
     * Version of the bot associated with the slot.
     */
    botVersion?: pulumi.Input<string>;
    /**
     * Description of the slot.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of the intent that contains the slot.
     */
    intentId?: pulumi.Input<string>;
    /**
     * Identifier of the language and locale that the slot will be used in.
     */
    localeId?: pulumi.Input<string>;
    /**
     * Whether the slot returns multiple values in one response.
     * See the `multipleValuesSetting` argument reference below.
     */
    multipleValuesSettings?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsSlotMultipleValuesSetting>[]>;
    /**
     * Name of the slot.
     */
    name?: pulumi.Input<string>;
    /**
     * Determines how slot values are used in Amazon CloudWatch logs.
     * See the `obfuscationSetting` argument reference below.
     */
    obfuscationSettings?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsSlotObfuscationSetting>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Unique identifier associated with the slot.
     */
    slotId?: pulumi.Input<string>;
    /**
     * Unique identifier for the slot type associated with this slot.
     */
    slotTypeId?: pulumi.Input<string>;
    /**
     * Specifications for the constituent sub slots and the expression for the composite slot.
     * See the `subSlotSetting` argument reference below.
     */
    subSlotSettings?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsSlotSubSlotSetting>[]>;
    timeouts?: pulumi.Input<inputs.lex.V2modelsSlotTimeouts>;
    valueElicitationSetting?: pulumi.Input<inputs.lex.V2modelsSlotValueElicitationSetting>;
}
/**
 * The set of arguments for constructing a V2modelsSlot resource.
 */
export interface V2modelsSlotArgs {
    /**
     * Identifier of the bot associated with the slot.
     */
    botId: pulumi.Input<string>;
    /**
     * Version of the bot associated with the slot.
     */
    botVersion: pulumi.Input<string>;
    /**
     * Description of the slot.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of the intent that contains the slot.
     */
    intentId: pulumi.Input<string>;
    /**
     * Identifier of the language and locale that the slot will be used in.
     */
    localeId: pulumi.Input<string>;
    /**
     * Whether the slot returns multiple values in one response.
     * See the `multipleValuesSetting` argument reference below.
     */
    multipleValuesSettings?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsSlotMultipleValuesSetting>[]>;
    /**
     * Name of the slot.
     */
    name?: pulumi.Input<string>;
    /**
     * Determines how slot values are used in Amazon CloudWatch logs.
     * See the `obfuscationSetting` argument reference below.
     */
    obfuscationSettings?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsSlotObfuscationSetting>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Unique identifier for the slot type associated with this slot.
     */
    slotTypeId?: pulumi.Input<string>;
    /**
     * Specifications for the constituent sub slots and the expression for the composite slot.
     * See the `subSlotSetting` argument reference below.
     */
    subSlotSettings?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsSlotSubSlotSetting>[]>;
    timeouts?: pulumi.Input<inputs.lex.V2modelsSlotTimeouts>;
    valueElicitationSetting?: pulumi.Input<inputs.lex.V2modelsSlotValueElicitationSetting>;
}

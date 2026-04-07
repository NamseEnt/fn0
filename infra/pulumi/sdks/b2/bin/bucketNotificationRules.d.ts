import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
export declare class BucketNotificationRules extends pulumi.CustomResource {
    /**
     * Get an existing BucketNotificationRules resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketNotificationRulesState, opts?: pulumi.CustomResourceOptions): BucketNotificationRules;
    /**
     * Returns true if the given object is an instance of BucketNotificationRules.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketNotificationRules;
    /**
     * The ID of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    readonly bucketId: pulumi.Output<string>;
    readonly bucketNotificationRulesId: pulumi.Output<string>;
    /**
     * An array of Event Notification Rules.
     */
    readonly notificationRules: pulumi.Output<outputs.BucketNotificationRulesNotificationRule[]>;
    /**
     * Create a BucketNotificationRules resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketNotificationRulesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketNotificationRules resources.
 */
export interface BucketNotificationRulesState {
    /**
     * The ID of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    bucketId?: pulumi.Input<string>;
    bucketNotificationRulesId?: pulumi.Input<string>;
    /**
     * An array of Event Notification Rules.
     */
    notificationRules?: pulumi.Input<pulumi.Input<inputs.BucketNotificationRulesNotificationRule>[]>;
}
/**
 * The set of arguments for constructing a BucketNotificationRules resource.
 */
export interface BucketNotificationRulesArgs {
    /**
     * The ID of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    bucketId: pulumi.Input<string>;
    bucketNotificationRulesId?: pulumi.Input<string>;
    /**
     * An array of Event Notification Rules.
     */
    notificationRules: pulumi.Input<pulumi.Input<inputs.BucketNotificationRulesNotificationRule>[]>;
}

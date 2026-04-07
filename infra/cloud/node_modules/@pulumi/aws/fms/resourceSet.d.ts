import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS FMS (Firewall Manager) Resource Set.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.fms.ResourceSet("example", {resourceSets: [{
 *     name: "testing",
 *     resourceTypeLists: ["AWS::NetworkFirewall::Firewall"],
 * }]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FMS (Firewall Manager) Resource Set using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fms/resourceSet:ResourceSet example resource_set-id-12345678
 * ```
 */
export declare class ResourceSet extends pulumi.CustomResource {
    /**
     * Get an existing ResourceSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceSetState, opts?: pulumi.CustomResourceOptions): ResourceSet;
    /**
     * Returns true if the given object is an instance of ResourceSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceSet;
    /**
     * ARN of the Resource Set.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Details about the resource set to be created or updated. See `resourceSet` Attribute Reference below.
     */
    readonly resourceSets: pulumi.Output<outputs.fms.ResourceSetResourceSet[] | undefined>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.fms.ResourceSetTimeouts | undefined>;
    /**
     * Create a ResourceSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ResourceSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceSet resources.
 */
export interface ResourceSetState {
    /**
     * ARN of the Resource Set.
     */
    arn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details about the resource set to be created or updated. See `resourceSet` Attribute Reference below.
     */
    resourceSets?: pulumi.Input<pulumi.Input<inputs.fms.ResourceSetResourceSet>[]>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.fms.ResourceSetTimeouts>;
}
/**
 * The set of arguments for constructing a ResourceSet resource.
 */
export interface ResourceSetArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details about the resource set to be created or updated. See `resourceSet` Attribute Reference below.
     */
    resourceSets?: pulumi.Input<pulumi.Input<inputs.fms.ResourceSetResourceSet>[]>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.fms.ResourceSetTimeouts>;
}

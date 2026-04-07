import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS WorkSpaces Connection Alias.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.workspaces.ConnectionAlias("example", {connectionString: "testdomain.test"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Connection Alias using the connection alias ID. For example:
 *
 * ```sh
 * $ pulumi import aws:workspaces/connectionAlias:ConnectionAlias example rft-8012925589
 * ```
 */
export declare class ConnectionAlias extends pulumi.CustomResource {
    /**
     * Get an existing ConnectionAlias resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectionAliasState, opts?: pulumi.CustomResourceOptions): ConnectionAlias;
    /**
     * Returns true if the given object is an instance of ConnectionAlias.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConnectionAlias;
    /**
     * The connection string specified for the connection alias. The connection string must be in the form of a fully qualified domain name (FQDN), such as www.example.com.
     */
    readonly connectionString: pulumi.Output<string>;
    /**
     * The identifier of the Amazon Web Services account that owns the connection alias.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The current state of the connection alias.
     */
    readonly state: pulumi.Output<string>;
    /**
     * A map of tags assigned to the WorkSpaces Connection Alias. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.workspaces.ConnectionAliasTimeouts | undefined>;
    /**
     * Create a ConnectionAlias resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectionAliasArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConnectionAlias resources.
 */
export interface ConnectionAliasState {
    /**
     * The connection string specified for the connection alias. The connection string must be in the form of a fully qualified domain name (FQDN), such as www.example.com.
     */
    connectionString?: pulumi.Input<string>;
    /**
     * The identifier of the Amazon Web Services account that owns the connection alias.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The current state of the connection alias.
     */
    state?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the WorkSpaces Connection Alias. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.workspaces.ConnectionAliasTimeouts>;
}
/**
 * The set of arguments for constructing a ConnectionAlias resource.
 */
export interface ConnectionAliasArgs {
    /**
     * The connection string specified for the connection alias. The connection string must be in the form of a fully qualified domain name (FQDN), such as www.example.com.
     */
    connectionString: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the WorkSpaces Connection Alias. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.workspaces.ConnectionAliasTimeouts>;
}

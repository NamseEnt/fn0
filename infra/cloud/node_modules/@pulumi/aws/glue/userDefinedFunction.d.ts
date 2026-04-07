import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue User Defined Function Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.CatalogDatabase("example", {name: "my_database"});
 * const exampleUserDefinedFunction = new aws.glue.UserDefinedFunction("example", {
 *     name: "my_func",
 *     catalogId: example.catalogId,
 *     databaseName: example.name,
 *     className: "class",
 *     ownerName: "owner",
 *     ownerType: "GROUP",
 *     resourceUris: [{
 *         resourceType: "ARCHIVE",
 *         uri: "uri",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue User Defined Functions using the `catalog_id:database_name:function_name`. If you have not set a Catalog ID specify the AWS Account ID that the database is in. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/userDefinedFunction:UserDefinedFunction func 123456789012:my_database:my_func
 * ```
 */
export declare class UserDefinedFunction extends pulumi.CustomResource {
    /**
     * Get an existing UserDefinedFunction resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserDefinedFunctionState, opts?: pulumi.CustomResourceOptions): UserDefinedFunction;
    /**
     * Returns true if the given object is an instance of UserDefinedFunction.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserDefinedFunction;
    /**
     * The ARN of the Glue User Defined Function.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the Glue Catalog to create the function in. If omitted, this defaults to the AWS Account ID.
     */
    readonly catalogId: pulumi.Output<string | undefined>;
    /**
     * The Java class that contains the function code.
     */
    readonly className: pulumi.Output<string>;
    /**
     * The time at which the function was created.
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * The name of the Database to create the Function.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * The name of the function.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The owner of the function.
     */
    readonly ownerName: pulumi.Output<string>;
    /**
     * The owner type. can be one of `USER`, `ROLE`, and `GROUP`.
     */
    readonly ownerType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The configuration block for Resource URIs. See resource uris below for more details.
     */
    readonly resourceUris: pulumi.Output<outputs.glue.UserDefinedFunctionResourceUri[] | undefined>;
    /**
     * Create a UserDefinedFunction resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserDefinedFunctionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserDefinedFunction resources.
 */
export interface UserDefinedFunctionState {
    /**
     * The ARN of the Glue User Defined Function.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the Glue Catalog to create the function in. If omitted, this defaults to the AWS Account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * The Java class that contains the function code.
     */
    className?: pulumi.Input<string>;
    /**
     * The time at which the function was created.
     */
    createTime?: pulumi.Input<string>;
    /**
     * The name of the Database to create the Function.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * The name of the function.
     */
    name?: pulumi.Input<string>;
    /**
     * The owner of the function.
     */
    ownerName?: pulumi.Input<string>;
    /**
     * The owner type. can be one of `USER`, `ROLE`, and `GROUP`.
     */
    ownerType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration block for Resource URIs. See resource uris below for more details.
     */
    resourceUris?: pulumi.Input<pulumi.Input<inputs.glue.UserDefinedFunctionResourceUri>[]>;
}
/**
 * The set of arguments for constructing a UserDefinedFunction resource.
 */
export interface UserDefinedFunctionArgs {
    /**
     * ID of the Glue Catalog to create the function in. If omitted, this defaults to the AWS Account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * The Java class that contains the function code.
     */
    className: pulumi.Input<string>;
    /**
     * The name of the Database to create the Function.
     */
    databaseName: pulumi.Input<string>;
    /**
     * The name of the function.
     */
    name?: pulumi.Input<string>;
    /**
     * The owner of the function.
     */
    ownerName: pulumi.Input<string>;
    /**
     * The owner type. can be one of `USER`, `ROLE`, and `GROUP`.
     */
    ownerType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration block for Resource URIs. See resource uris below for more details.
     */
    resourceUris?: pulumi.Input<pulumi.Input<inputs.glue.UserDefinedFunctionResourceUri>[]>;
}

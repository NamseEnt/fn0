import { Input, Inputs, Output } from "./output";
export declare type ID = string;
export declare type URN = string;
/**
 * {@link createUrn} computes a URN from the combination of a resource name,
 * resource type, optional parent, optional project and optional stack.
 */
export declare function createUrn(name: Input<string>, type: Input<string>, parent?: Resource | Input<URN>, project?: string, stack?: string): Output<string>;
/**
 * {@link allAliases} computes the full set of aliases for a child resource
 * given a set of aliases applied to the child and parent resources. This
 * includes the child resource's own aliases, as well as aliases inherited from
 * the parent. If there are N child aliases, and M parent aliases, there will be
 * (M+1)*(N+1)-1 total aliases, or, as calculated in the logic below,
 * N+(M*(1+N)).
 */
export declare function allAliases(childAliases: Input<URN | Alias>[], childName: string, childType: string, parent: Resource, parentName: string): Output<URN>[];
/**
 * {@link Resource} represents a class whose CRUD operations are implemented by
 * a provider plugin.
 */
export declare abstract class Resource {
    /**
     * A regexp for use with {@link sourcePosition}.
     */
    private static framePositionRegExp;
    /**
     * The stable logical URN used to distinctly address a resource, both before
     * and after deployments.
     */
    readonly urn: Output<URN>;
    static isInstance(obj: any): obj is Resource;
    /**
     * Returns the list of callsites for the current stack. The top of the stack is at index 0.
     *
     * Some callsites may not contain position information. It is up to the caller to detect this situation.
     */
    private static callsites;
    private static stackTrace;
    /**
     * Returns the provider for the given module member, if one exists.
     */
    getProvider(moduleMember: string): ProviderResource | undefined;
    /**
     * Creates and registers a new resource object. `t` is the fully qualified
     * type token and `name` is the "name" part to use in creating a stable and
     * globally unique URN for the object. `dependsOn` is an optional list of
     * other resources that this resource depends on, controlling the order in
     * which we perform resource operations.
     *
     * @param t
     *  The type of the resource.
     * @param name
     *  The _unique_ name of the resource.
     * @param custom
     *  True to indicate that this is a custom resource, managed by a plugin.
     * @param props
     *  The arguments to use to populate the new resource.
     * @param opts
     *  A bag of options that control this resource's behavior.
     * @param remote
     *  True if this is a remote component resource.
     * @param dependency
     *  True if this is a synthetic resource used internally for dependency tracking.
     */
    constructor(t: string, name: string, custom: boolean, props?: Inputs, opts?: ResourceOptions, remote?: boolean, dependency?: boolean, packageRef?: Promise<string | undefined>);
}
/**
 * A constant to represent the "root stack" resource for a Pulumi application.
 * The purpose of this is solely to make it easy to write an {@link Alias} like
 * so:
 *
 * `aliases: [{ parent: rootStackResource }]`.
 *
 * This indicates that the prior name for a resource was created based on it
 * being parented directly by the stack itself and no other resources. Note:
 * this is equivalent to:
 *
 * `aliases: [{ parent: undefined }]`
 *
 * However, the former form is preferable as it is more self-descriptive, while
 * the latter may look a bit confusing and may incorrectly look like something
 * that could be removed without changing semantics.
 */
export declare const rootStackResource: Resource;
/**
 * {@link Alias} is a partial description of prior names used for a resource. It
 * can be processed in the context of a resource creation to determine what the
 * full aliased URN would be.
 *
 * Note there is a semantic difference between properties being absent from this
 * type and properties having the `undefined` value. Specifically, there is a
 * difference between:
 *
 * ```ts
 * { name: "foo", parent: undefined } // and
 * { name: "foo" }
 * ```
 *
 * The presence of a property indicates if its value should be used.  If absent,
 * then the value is not used. So, in the above while `alias.parent` is
 * `undefined` for both, the first alias means "the original URN had no parent"
 * while the second alias means "use the current parent".
 *
 * Note: to indicate that a resource was previously parented by the root stack,
 * it is recommended that you use:
 *
 * `aliases: [{ parent: pulumi.rootStackResource }]`
 *
 * This form is self-descriptive and makes the intent clearer than using:
 *
 * `aliases: [{ parent: undefined }]`
 */
export interface Alias {
    /**
     * The previous name of the resource. If not provided, the current name of
     * the resource is used.
     */
    name?: Input<string>;
    /**
     * The previous type of the resource. If not provided, the current type of
     * the resource is used.
     */
    type?: Input<string>;
    /**
     * The previous parent of the resource. If not provided (i.e. `{ name: "foo"
     * }`), the current parent of the resource is used (`opts.parent` if
     * provided, else the implicit stack resource parent).
     *
     * To specify no original parent, use `{ parent: pulumi.rootStackResource }`.
     */
    parent?: Resource | Input<URN>;
    /**
     * The previous stack of the resource. If not provided, defaults to
     * `pulumi.getStack()`.
     */
    stack?: Input<string>;
    /**
     * The previous project of the resource. If not provided, defaults to
     * `pulumi.getProject()`.
     */
    project?: Input<string>;
}
/**
 * {@link ResourceOptions} is a bag of optional settings that control a
 * resource's behavior.
 */
export interface ResourceOptions {
    /**
     * An optional existing ID to load, rather than create.
     */
    id?: Input<ID>;
    /**
     * An optional parent resource to which this resource belongs.
     */
    parent?: Resource;
    /**
     * An optional additional explicit dependencies on other resources.
     */
    dependsOn?: Input<Input<Resource>[]> | Input<Resource>;
    /**
     * When set to true, protect ensures this resource cannot be deleted.
     */
    protect?: boolean;
    /**
     * Ignore changes to any of the specified properties.
     */
    ignoreChanges?: string[];
    /**
     * Changes to any of these property paths will force a replacement.  If this list includes `"*"`, changes to any
     * properties will force a replacement.  Initialization errors from previous deployments will require replacement
     * instead of update only if `"*"` is passed.
     */
    replaceOnChanges?: string[];
    /**
     * An optional version, corresponding to the version of the provider plugin that should be used when operating on
     * this resource. This version overrides the version information inferred from the current package and should
     * rarely be used.
     */
    version?: string;
    /**
     * An optional list of aliases to treat this resource as matching.
     */
    aliases?: Input<URN | Alias>[];
    /**
     * An optional provider to use for this resource's CRUD operations. If no provider is supplied,
     * the default provider for the resource's package will be used. The default provider is pulled
     * from the parent's provider bag (see also ComponentResourceOptions.providers).
     *
     * If this is a [ComponentResourceOptions] do not provide both [provider] and [providers]
     */
    provider?: ProviderResource;
    /**
     * An optional customTimeouts configuration block.
     */
    customTimeouts?: CustomTimeouts;
    /**
     * Optional list of transformations to apply to this resource during construction. The
     * transformations are applied in order, and are applied prior to transformation applied to
     * parents walking from the resource up to the stack.
     */
    transformations?: ResourceTransformation[];
    /**
     * Optional list of transforms to apply to this resource during construction. The
     * transforms are applied in order, and are applied prior to transforms applied to
     * parents walking from the resource up to the stack.
     *
     * This property is experimental.
     */
    transforms?: ResourceTransform[];
    /**
     * The URN of a previously-registered resource of this type to read from the engine.
     */
    urn?: URN;
    /**
     * An option to specify the URL from which to download this resources
     * associated plugin. This version overrides the URL information inferred
     * from the current package and should rarely be used.
     */
    pluginDownloadURL?: string;
    /**
     * If set to True, the providers Delete method will not be called for this resource.
     */
    retainOnDelete?: boolean;
    /**
     * If set, the providers Delete method will not be called for this resource
     * if specified is being deleted as well.
     */
    deletedWith?: Resource;
    /**
     * If set, the URNs of the resources whose replaces will also replace this resource.
     */
    replaceWith?: Resource[];
    /**
     * Optional resource hooks to bind to this resource. The hooks will be
     * invoked during certain step of the lifecycle of the resource.
     */
    hooks?: ResourceHookBinding;
    /**
     * A list of property paths where the diffs will be hidden. This only changes display logic.
     */
    hideDiffs?: string[];
}
export interface CustomTimeouts {
    /**
     * The optional create timeout represented as a string e.g. 5m, 40s, 1d.
     */
    create?: string;
    /**
     * The optional update timeout represented as a string e.g. 5m, 40s, 1d.
     */
    update?: string;
    /**
     * The optional delete timeout represented as a string e.g. 5m, 40s, 1d.
     */
    delete?: string;
}
/**
 * {@link ResourceTransformation} is the callback signature for the
 * `transformations` resource option.  A transformation is passed the same set
 * of inputs provided to the {@link Resource} constructor, and can optionally
 * return back alternate values for the `props` and/or `opts` prior to the
 * resource actually being created.  The effect will be as though those props
 * and opts were passed in place of the original call to the {@link Resource}
 * constructor. If the transformation returns `undefined`, this indicates that
 * the resource will not be transformed.
 */
export declare type ResourceTransformation = (args: ResourceTransformationArgs) => ResourceTransformationResult | undefined;
/**
 * {@link ResourceTransform} is the callback signature for the `transforms`
 * resource option.  A transform is passed the same set of inputs provided to
 * the {@link Resource} constructor, and can optionally return back alternate
 * values for the `props` and/or `opts` prior to the resource actually being
 * created.  The effect will be as though those props and opts were passed in
 * place of the original call to the {@link Resource} constructor.  If the
 * transform returns `undefined`, this indicates that the resource will not be
 * transformed.
 */
export declare type ResourceTransform = (args: ResourceTransformArgs) => Promise<ResourceTransformResult | undefined> | ResourceTransformResult | undefined;
/**
 * {@link ResourceTransformArgs} is the argument bag passed to a resource transform.
 */
export interface ResourceTransformArgs {
    /**
     * True if the resource is a custom resource, false if it is a component resource.
     */
    custom: boolean;
    /**
     * The type of the resource.
     */
    type: string;
    /**
     * The name of the resource.
     */
    name: string;
    /**
     * The original properties passed to the resource constructor.
     */
    props: Inputs;
    /**
     * The original resource options passed to the resource constructor.
     */
    opts: ResourceOptions;
}
/**
 * {@link ResourceTransformResult} is the result that must be returned by a
 * resource transform callback.  It includes new values to use for the
 * `props` and `opts` of the {@link Resource} in place of the originally
 * provided values.
 */
export interface ResourceTransformResult {
    /**
     * The new properties to use in place of the original `props`
     */
    props: Inputs;
    /**
     * The new resource options to use in place of the original `opts`
     */
    opts: ResourceOptions;
}
/**
 * {@link ResourceTransformationArgs} is the argument bag passed to a resource
 * transformation.
 */
export interface ResourceTransformationArgs {
    /**
     * The {@link Resource} instance that is being transformed.
     */
    resource: Resource;
    /**
     * The type of the resource.
     */
    type: string;
    /**
     * The name of the resource.
     */
    name: string;
    /**
     * The original properties passed to the resource constructor.
     */
    props: Inputs;
    /**
     * The original resource options passed to the resource constructor.
     */
    opts: ResourceOptions;
}
/**
 * {@link ResourceTransformationResult} is the result that must be returned by a
 * resource transformation callback.  It includes new values to use for the
 * `props` and `opts` of the {@link Resource} in place of the originally
 * provided values.
 */
export interface ResourceTransformationResult {
    /**
     * The new properties to use in place of the original `props`
     */
    props: Inputs;
    /**
     * The new resource options to use in place of the original `opts`
     */
    opts: ResourceOptions;
}
/**
 * {@link CustomResourceOptions} is a bag of optional settings that control a
 * custom resource's behavior.
 */
export interface CustomResourceOptions extends ResourceOptions {
    /**
     * When set to true, indicates that this resource should be deleted before
     * its replacement is created when replacement is necessary.
     */
    deleteBeforeReplace?: boolean;
    /**
     * The names of outputs for this resource that should be treated as secrets.
     * This augments the list that the resource provider and Pulumi engine
     * already determine based on inputs to your resource. It can be used to
     * mark certain ouputs as a secrets on a per resource basis.
     */
    additionalSecretOutputs?: string[];
    /**
     * When provided with a resource ID, indicates that this resource's provider
     * should import its state from the cloud resource with the given ID. The
     * inputs to the resource's constructor must align with the resource's
     * current state. Once a resource has been imported, the import property
     * must be removed from the resource's options.
     */
    import?: ID;
}
/**
 * ResourceHook is a named hook that can be registered as a resource hook.
 */
export declare class ResourceHook {
    /**
     * The unqiue name of the resource hook.
     */
    name: string;
    /**
     * The function that will be called when the resource hook is triggered.
     */
    callback: ResourceHookFunction;
    /**
     * Run the hook during dry run (preview) operations. Defaults to false.
     */
    opts?: ResourceHookOptions;
    constructor(name: string, callback: ResourceHookFunction, opts?: ResourceHookOptions);
    static isInstance(obj: any): obj is ResourceHook;
}
/**
 * Options for registering a resource hook.
 */
export interface ResourceHookOptions {
    /**
     * Run the hook during dry run (preview) operations. Defaults to false.
     */
    onDryRun?: boolean;
}
/**
 * ResourceHookArgs represents the arguments passed to a resource hook
 * Depending on the hook type, only some of the new/old inputs/outputs are set.
 *
 * | Hook Type     | old_inputs | new_inputs | old_outputs | new_outputs |
 * | ------------- | ---------- | ---------- | ----------- | ----------- |
 * | before_create |            | ✓          |             |             |
 * | after_create  |            | ✓          |             | ✓           |
 * | before_update | ✓          | ✓          | ✓           |             |
 * | after_update  | ✓          | ✓          | ✓           | ✓           |
 * | before_delete | ✓          |            | ✓           |             |
 * | after_delete  | ✓          |            | ✓           |             |
 */
export interface ResourceHookArgs {
    /**
     * The URN of the resource that triggered the hook.
     */
    urn: URN;
    /**
     * The ID of the resource that triggered the hook.
     */
    id: ID;
    /**
     * The name of the resource that triggered the hook.
     */
    name: string;
    /**
     * The type of the resource that triggered the hook.
     */
    type: string;
    /**
     * The new inputs of the resource that triggered the hook.
     */
    newInputs?: Record<string, any>;
    /**
     * The old inputs of the resource that triggered the hook.
     */
    oldInputs?: Record<string, any>;
    /**
     * The new outputs of the resource that triggered the hook.
     */
    newOutputs?: Record<string, any>;
    /**
     * The old outputs of the resource that triggered the hook.
     */
    oldOutputs?: Record<string, any>;
}
/**
 * ResourceHookFunction is a function that can be registered as a resource hook.
 */
export declare type ResourceHookFunction = (args: ResourceHookArgs) => void | Promise<void>;
/**
 * Binds {@link ResourceHook} instances to a resource. The resource hooks will
 * be invoked during certain step of the lifecycle of the resource.
 *
 * `before_${action}` hooks that raise an exception cause the action to fail.
 * `after_${action}` hooks that raise an exception will log a warning, but do
 * not cause the action or the deployment to fail.
 *
 * When running `pulumi destroy`, `before_delete` and `after_delete` resource
 * hooks require the operation to run with `--run-program`, to ensure that the
 * program which defines the hooks is available.
 */
export interface ResourceHookBinding {
    /**
     * Hooks to be invoked before the resource is created.
     */
    beforeCreate?: Array<ResourceHookFunction | ResourceHook>;
    /**
     * Hooks to be invoked after the resource is created.
     */
    afterCreate?: Array<ResourceHookFunction | ResourceHook>;
    /**
     * Hooks to be invoked before the resource is updated.
     */
    beforeUpdate?: Array<ResourceHookFunction | ResourceHook>;
    /**
     * Hooks to be invoked after the resource is updated.
     */
    afterUpdate?: Array<ResourceHookFunction | ResourceHook>;
    /**
     * Hooks to be invoked before the resource is deleted.
     *
     * Note that delete hooks require that destroy operations are run with `--run-program`. Unlike other hook types,
     * this argument requires named {@link ResourceHook} instances, and does not accept anonymous
     * {@link ResourceHookFunction}. This is because the engine needs to be able to identify a hook when a resource is
     * deleted.
     */
    beforeDelete?: Array<ResourceHook>;
    /**
     * Hooks to be invoked after the resource is deleted.
     *
     * Note that delete hooks require that destroy operations are run with `--run-program`. Unlike other hook types,
     * this argument requires named {@link ResourceHook} instances, and does not accept anonymous
     * {@link ResourceHookFunction}. This is because the engine needs to be able to identify a hook when a resource is
     * deleted.
     */
    afterDelete?: Array<ResourceHook>;
}
/**
 * {@link ComponentResourceOptions} is a bag of optional settings that control a
 * component resource's behavior.
 */
export interface ComponentResourceOptions extends ResourceOptions {
    /**
     * An optional set of providers to use for child resources. Either keyed by
     * package name (e.g. "aws"), or just provided as an array.  In the latter
     * case, the package name will be retrieved from the provider itself.
     *
     * Note: only a list should be used. Mapping keys are not respected.
     */
    providers?: Record<string, ProviderResource> | ProviderResource[];
}
/**
 * {@link CustomResource} is a resource whose create, read, update, and delete
 * (CRUD) operations are managed by performing external operations on some
 * physical entity.  The engine understands how to diff and perform partial
 * updates of them, and these CRUD operations are implemented in a dynamically
 * loaded plugin for the defining package.
 */
export declare abstract class CustomResource extends Resource {
    /**
     * The provider-assigned unique ID for this managed resource. It is set
     * during deployments and may be missing (undefined) during planning phases.
     */
    readonly id: Output<ID>;
    /**
     * Returns true if the given object is a {@link CustomResource}. This is
     * designed to work even when multiple copies of the Pulumi SDK have been
     * loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomResource;
    /**
     * Creates and registers a new managed resource. `t` is the fully qualified
     * type token and `name` is the "name" part to use in creating a stable and
     * globally unique URN for the object. `dependsOn` is an optional list of
     * other resources that this resource depends on, controlling the order in
     * which we perform resource operations. Creating an instance does not
     * necessarily perform a create on the physical entity which it represents.
     * Instead, this is dependent upon the diffing of the new goal state
     * compared to the current known resource state.
     *
     * @param t
     *  The type of the resource.
     * @param name
     *  The _unique_ name of the resource.
     * @param props
     *  The arguments to use to populate the new resource.
     * @param opts
     *  A bag of options that control this resource's behavior.
     * @param dependency
     *  True if this is a synthetic resource used internally for dependency tracking.
     */
    constructor(t: string, name: string, props?: Inputs, opts?: CustomResourceOptions, dependency?: boolean, packageRef?: Promise<string | undefined>);
}
/**
 * {@link ProviderResource} is a resource that implements CRUD operations for
 * other custom resources. These resources are managed similarly to other
 * resources, including the usual diffing and update semantics.
 */
export declare abstract class ProviderResource extends CustomResource {
    static register(provider: ProviderResource | undefined): Promise<string | undefined>;
    /**
     * Creates and registers a new provider resource for a particular package.
     *
     * @param pkg
     *  The package associated with this provider.
     * @param name
     *  The _unique_ name of the provider.
     * @param props
     *  The configuration to use for this provider.
     * @param opts
     *  A bag of options that control this provider's behavior.
     * @param dependency
     *  True if this is a synthetic resource used internally for dependency tracking.
     */
    constructor(pkg: string, name: string, props?: Inputs, opts?: ResourceOptions, dependency?: boolean, packageRef?: Promise<string | undefined>);
}
/**
 * {@link ComponentResource} is a resource that aggregates one or more other
 * child resources into a higher level abstraction. The component resource
 * itself is a resource, but does not require custom CRUD operations for
 * provisioning.
 */
export declare class ComponentResource<TData = any> extends Resource {
    /**
     * Returns true if the given object is a {@link CustomResource}. This is
     * designed to work even when multiple copies of the Pulumi SDK have been
     * loaded into the same process.
     */
    static isInstance(obj: any): obj is ComponentResource;
    /**
     * Creates and registers a new component resource. `type` is the fully
     * qualified type token and `name` is the "name" part to use in creating a
     * stable and globally unique URN for the object. `opts.parent` is the
     * optional parent for this component, and `opts.dependsOn` is an optional
     * list of other resources that this resource depends on, controlling the
     * order in which we perform resource operations.
     *
     * @param type
     *  The type of the resource.
     * @param name
     *  The _unique_ name of the resource.
     * @param args
     *  Information passed to [initialize] method.
     * @param opts
     *  A bag of options that control this resource's behavior.
     * @param remote
     *  True if this is a remote component resource.
     */
    constructor(type: string, name: string, args?: Inputs, opts?: ComponentResourceOptions, remote?: boolean, packageRef?: Promise<string | undefined>);
    /**
     * Can be overridden by a subclass to asynchronously initialize data for this component
     * automatically when constructed. The data will be available immediately for subclass
     * constructors to use. To access the data use {@link getData}.
     */
    protected initialize(args: Inputs, opts?: ComponentResourceOptions, name?: string, type?: string): Promise<TData>;
    /**
     * Retrieves the data produces by {@link initialize}. The data is
     * immediately available in a derived class's constructor after the
     * `super(...)` call to `ComponentResource`.
     */
    protected getData(): Promise<TData>;
    /**
     * Registers synthetic outputs that a component has initialized, usually by
     * allocating other child sub-resources and propagating their resulting
     * property values.
     *
     * Component resources can call this at the end of their constructor to
     * indicate that they are done creating child resources.  This is not
     * strictly necessary as this will automatically be called after the {@link
     * initialize} method completes.
     */
    protected registerOutputs(outputs?: Inputs | Promise<Inputs> | Output<Inputs>): void;
}
/**
 * {@link mergeOptions} takes two {@link ResourceOptions} values and produces a new
 * {@link ResourceOptions} with the respective properties of `opts2` merged over the
 * same properties in `opts1`. The original options objects will be unchanged.
 *
 * Conceptually property merging follows these basic rules:
 *
 *  1. if the property is a collection, the final value will be a collection
 *     containing the values from each options object.
 *
 *  2. Simple scaler values from `opts2` (i.e. strings, numbers, bools) will
 *     replace the values of `opts1`.
 *
 *  3. `opts2` can have properties explicitly provided with `null` or
 *     `undefined` as the value. If explicitly provided, then that will be the
 *     final value in the result.
 *
 *  4. For the purposes of merging `dependsOn`, `provider` and `providers` are
 *     always treated as collections, even if only a single value was provided.
 */
export declare function mergeOptions(opts1: CustomResourceOptions | undefined, opts2: CustomResourceOptions | undefined): CustomResourceOptions;
export declare function mergeOptions(opts1: ComponentResourceOptions | undefined, opts2: ComponentResourceOptions | undefined): ComponentResourceOptions;
export declare function mergeOptions(opts1: ResourceOptions | undefined, opts2: ResourceOptions | undefined): ResourceOptions;
/**
 * A {@link DependencyResource} is a resource that is used to indicate that an
 * {@link Output} has a dependency on a particular resource. These resources are
 * only created when dealing with remote component resources.
 */
export declare class DependencyResource extends CustomResource {
    constructor(urn: URN);
}
/**
 * A {@link DependencyProviderResource} is a resource that is used by the
 * provider SDK as a stand-in for a provider that is only used for its
 * reference. Its only valid properties are its URN and ID.
 */
export declare class DependencyProviderResource extends ProviderResource {
    constructor(ref: string);
}
/**
 * The Pulumi type assigned to the resource at construction, of the form `package:module:name`.
 */
export declare function resourceType(res: Resource): string;
/**
 * The Pulumi name assigned to the resource at construction, i.e. the "name" in its constructor call.
 */
export declare function resourceName(res: Resource): string;

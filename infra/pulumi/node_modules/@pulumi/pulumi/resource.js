"use strict";
// Copyright 2016-2025, Pulumi Corporation.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const url = __importStar(require("url"));
const errors_1 = require("./errors");
const log = __importStar(require("./log"));
const output_1 = require("./output");
const resource_1 = require("./runtime/resource");
const rpc_1 = require("./runtime/rpc");
const settings_1 = require("./runtime/settings");
const state_1 = require("./runtime/state");
const utils = __importStar(require("./utils"));
/**
 * {@link createUrn} computes a URN from the combination of a resource name,
 * resource type, optional parent, optional project and optional stack.
 */
function createUrn(name, type, parent, project, stack) {
    let parentPrefix;
    if (parent) {
        let parentUrn;
        if (Resource.isInstance(parent)) {
            parentUrn = parent.urn;
        }
        else {
            parentUrn = output_1.output(parent);
        }
        parentPrefix = parentUrn.apply((parentUrnString) => {
            const prefix = parentUrnString.substring(0, parentUrnString.lastIndexOf("::")) + "$";
            if (prefix.endsWith("::pulumi:pulumi:Stack$")) {
                // Don't prefix the stack type as a parent type
                return `urn:pulumi:${stack || settings_1.getStack()}::${project || settings_1.getProject()}::`;
            }
            return prefix;
        });
    }
    else {
        parentPrefix = output_1.output(`urn:pulumi:${stack || settings_1.getStack()}::${project || settings_1.getProject()}::`);
    }
    return output_1.interpolate `${parentPrefix}${type}::${name}`;
}
exports.createUrn = createUrn;
/**
 * {@link inheritedChildAlias} computes the alias that should be applied to a
 * child based on an alias applied to its parent. This may involve changing the
 * name of the resource in cases where the resource has a named derived from the
 * name of the parent, and the parent name changed.
 */
function inheritedChildAlias(childName, parentName, parentAlias, childType) {
    // If the child name has the parent name as a prefix, then we make the assumption that it was
    // constructed from the convention of using `{name}-details` as the name of the child resource.  To
    // ensure this is aliased correctly, we must then also replace the parent aliases name in the prefix of
    // the child resource name.
    //
    // For example:
    // * name: "newapp-function"
    // * opts.parent.__name: "newapp"
    // * parentAlias: "urn:pulumi:stackname::projectname::awsx:ec2:Vpc::app"
    // * parentAliasName: "app"
    // * aliasName: "app-function"
    // * childAlias: "urn:pulumi:stackname::projectname::aws:s3/bucket:Bucket::app-function"
    let aliasName = output_1.output(childName);
    if (childName.startsWith(parentName)) {
        aliasName = output_1.output(parentAlias).apply((parentAliasUrn) => {
            const parentAliasName = parentAliasUrn.substring(parentAliasUrn.lastIndexOf("::") + 2);
            return parentAliasName + childName.substring(parentName.length);
        });
    }
    return createUrn(aliasName, childType, parentAlias);
}
/**
 * Extracts the type and name from a URN.
 */
function urnTypeAndName(urn) {
    const parts = urn.split("::");
    const typeParts = parts[2].split("$");
    return {
        name: parts[3],
        type: typeParts[typeParts.length - 1],
    };
}
/**
 * {@link allAliases} computes the full set of aliases for a child resource
 * given a set of aliases applied to the child and parent resources. This
 * includes the child resource's own aliases, as well as aliases inherited from
 * the parent. If there are N child aliases, and M parent aliases, there will be
 * (M+1)*(N+1)-1 total aliases, or, as calculated in the logic below,
 * N+(M*(1+N)).
 */
function allAliases(childAliases, childName, childType, parent, parentName) {
    const aliases = [];
    for (const childAlias of childAliases) {
        aliases.push(collapseAliasToUrn(childAlias, childName, childType, parent));
    }
    for (const parentAlias of parent.__aliases || []) {
        // For each parent alias, add an alias that uses that base child name and the parent alias
        aliases.push(inheritedChildAlias(childName, parentName, parentAlias, childType));
        // Also add an alias for each child alias and the parent alias
        for (const childAlias of childAliases) {
            const inheritedAlias = collapseAliasToUrn(childAlias, childName, childType, parent).apply((childAliasURN) => {
                const { name: aliasedChildName, type: aliasedChildType } = urnTypeAndName(childAliasURN);
                return inheritedChildAlias(aliasedChildName, parentName, parentAlias, aliasedChildType);
            });
            aliases.push(inheritedAlias);
        }
    }
    return aliases;
}
exports.allAliases = allAliases;
/**
 * {@link Resource} represents a class whose CRUD operations are implemented by
 * a provider plugin.
 */
class Resource {
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
    constructor(t, name, custom, props = {}, opts = {}, remote = false, dependency = false, packageRef) {
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention,no-underscore-dangle,id-blacklist,id-match
        this.__pulumiResource = true;
        this.__pulumiType = t;
        if (dependency) {
            this.__providers = {};
            return;
        }
        if (opts.parent && !Resource.isInstance(opts.parent)) {
            throw new Error(`Resource parent is not a valid Resource: ${opts.parent}`);
        }
        if (!t) {
            throw new errors_1.ResourceError("Missing resource type argument", opts.parent);
        }
        if (!name) {
            throw new errors_1.ResourceError("Missing resource name argument (for URN creation)", opts.parent);
        }
        // Before anything else - if there are transformations registered, invoke them in order to transform the properties and
        // options assigned to this resource.
        const parent = opts.parent || state_1.getStackResource();
        this.__transformations = [...(opts.transformations || []), ...(parent?.__transformations || [])];
        for (const transformation of this.__transformations) {
            const tres = transformation({ resource: this, type: t, name, props, opts });
            if (tres) {
                if (tres.opts.parent !== opts.parent) {
                    // This is currently not allowed because the parent tree is needed to establish what
                    // transformation to apply in the first place, and to compute inheritance of other
                    // resource options in the Resource constructor before transformations are run (so
                    // modifying it here would only even partially take affect).  It's theoretically
                    // possible this restriction could be lifted in the future, but for now just
                    // disallow re-parenting resources in transformations to be safe.
                    throw new Error("Transformations cannot currently be used to change the `parent` of a resource.");
                }
                props = tres.props;
                opts = tres.opts;
            }
        }
        this.__name = name;
        // Make a shallow clone of opts to ensure we don't modify the value passed in.
        opts = Object.assign({}, opts);
        // Check the parent type if one exists and fill in any default options.
        this.__providers = {};
        if (parent) {
            this.__parentResource = parent;
            this.__parentResource.__childResources = this.__parentResource.__childResources || new Set();
            this.__parentResource.__childResources.add(this);
            if (opts.protect === undefined) {
                opts.protect = parent.__protect;
            }
            this.__providers = parent.__providers;
        }
        // providers is found by combining (in ascending order of priority)
        //      1. provider
        //      2. self_providers
        //      3. opts.providers
        this.__providers = {
            ...this.__providers,
            ...convertToProvidersMap(opts.providers),
            ...convertToProvidersMap(opts.provider ? [opts.provider] : {}),
        };
        const pkg = pkgFromType(t);
        // provider is the first option that does not return none
        // 1. opts.provider
        // 2. a matching provider in opts.providers
        // 3. a matching provider inherited from opts.parent
        if ((custom || remote) && opts.provider === undefined) {
            const parentProvider = parent?.getProvider(t);
            if (pkg && pkg in this.__providers) {
                opts.provider = this.__providers[pkg];
            }
            else if (parentProvider) {
                opts.provider = parentProvider;
            }
        }
        // Custom and remote resources have a backing provider. If this is a custom or
        // remote resource and a provider has been specified that has the same package
        // as the resource's package, save it in `__prov`.
        // If the provider's package isn't the same as the resource's package, don't
        // save it in `__prov` because the user specified `provider: someProvider` as
        // shorthand for `providers: [someProvider]`, which is a provider intended for
        // the component's children and not for this resource itself.
        // `__prov` is passed along in `Call` gRPC requests for resource method calls
        // (when set) so that the call goes to the same provider as the resource.
        if ((custom || remote) && opts.provider) {
            if (pkg && pkg === opts.provider.getPackage()) {
                this.__prov = opts.provider;
            }
        }
        this.__protect = opts.protect;
        this.__version = opts.version;
        this.__pluginDownloadURL = opts.pluginDownloadURL;
        // Collapse any `Alias`es down to URNs. We have to wait until this point to do so because we do not know the
        // default `name` and `type` to apply until we are inside the resource constructor.
        this.__aliases = [];
        if (opts.aliases) {
            for (const alias of opts.aliases) {
                this.__aliases.push(collapseAliasToUrn(alias, name, t, parent));
            }
        }
        // This is somewhat brittle in that it expects a call stack of the form:
        //
        // - {@link Resource} class constructor
        // - abstract {@link Resource} subclass constructor
        // - concrete {@link Resource} subclass constructor
        // - user code
        //
        // This stack reflects the expected class hierarchy of:
        //
        // Resource > Custom/Component resource > Cloud/Component resource
        //
        // For example, consider the AWS S3 Bucket resource. When user code
        // instantiates a Bucket, the stack will look like this:
        //
        //     new Resource (/path/to/resource.ts:123:45)
        //     new CustomResource (/path/to/resource.ts:678:90)
        //     new Bucket (/path/to/bucket.ts:987:65)
        //     <user code> (/path/to/index.ts:4:3)
        const stackTrace = Resource.stackTrace(4);
        const sourcePosition = stackTrace.length < 1 ? undefined : stackTrace[0];
        if (opts.urn) {
            // This is a resource that already exists. Read its state from the engine.
            resource_1.getResource(this, parent, props, custom, opts.urn);
        }
        else if (opts.id) {
            // If this is a custom resource that already exists, read its state from the provider.
            if (!custom) {
                throw new errors_1.ResourceError("Cannot read an existing resource unless it has a custom provider", opts.parent);
            }
            resource_1.readResource(this, parent, t, name, props, opts, sourcePosition, stackTrace, packageRef);
        }
        else {
            // Kick off the resource registration.  If we are actually performing a deployment, this
            // resource's properties will be resolved asynchronously after the operation completes, so
            // that dependent computations resolve normally.  If we are just planning, on the other
            // hand, values will never resolve.
            resource_1.registerResource(this, parent, t, name, custom, remote, (urn) => new DependencyResource(urn), props, opts, sourcePosition, stackTrace, packageRef);
        }
    }
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiResource");
    }
    /**
     * Returns the list of callsites for the current stack. The top of the stack is at index 0.
     *
     * Some callsites may not contain position information. It is up to the caller to detect this situation.
     */
    static callsites() {
        // NOTE: it would be lovely to implement this using prepareStackTrace and actual V8 callsite information.
        // Unfortunately, the V8 source locations are not mapped from transpiled locations to user locations, and
        // Node 20.x does not expose an easy way to access source map information. Furthermore, ts-node may inject
        // hooks that perform this mapping as part of prepareStackTrace. Instead of using prepareStackTrace, then,
        // we parse its output as returned via `new Error().stack`. The stack trace will look like:
        //
        //     Error
        //         at foo (file:line:column)
        //         at bar (file:line:column)
        //         at baz (file:line:column)
        //
        // We process this by splitting it into lines and parsing the position information out of each line.
        const stackTraceLimit = Error.stackTraceLimit;
        try {
            Error.stackTraceLimit = Number.POSITIVE_INFINITY;
            const stackTrace = new Error().stack;
            return (stackTrace
                ?.split("\n")
                ?.slice(1)
                ?.map((frame) => {
                const { file, line, column } = Resource.framePositionRegExp.exec(frame)?.groups || {};
                if (!file || !line || !column || file.startsWith("node:")) {
                    return {};
                }
                const lineNum = parseInt(line, 10);
                const colNum = parseInt(column, 10);
                if (Number.isNaN(lineNum) || Number.isNaN(colNum)) {
                    return {};
                }
                return { file, line: lineNum, column: colNum };
            }) || []);
        }
        finally {
            Error.stackTraceLimit = stackTraceLimit;
        }
    }
    static stackTrace(skip) {
        let stack = Resource.callsites();
        if (stack.length < skip + 1) {
            return [];
        }
        stack = stack.slice(skip + 1);
        return stack.map((frame) => {
            const { file, line, column } = frame;
            if (!file || !line || !column) {
                return undefined;
            }
            return {
                uri: url.pathToFileURL(file).toString(),
                line,
                column,
            };
        });
    }
    /**
     * Returns the provider for the given module member, if one exists.
     */
    getProvider(moduleMember) {
        const pkg = pkgFromType(moduleMember);
        if (pkg === undefined) {
            return undefined;
        }
        return this.__providers[pkg];
    }
}
exports.Resource = Resource;
/**
 * A regexp for use with {@link sourcePosition}.
 */
Resource.framePositionRegExp = /^\s*at .* \((?<file>.*):(?<line>[0-9]+):(?<column>[0-9]+)\)$/;
function convertToProvidersMap(providers) {
    if (!providers) {
        return {};
    }
    if (!Array.isArray(providers)) {
        return providers;
    }
    const result = {};
    for (const provider of providers) {
        result[provider.getPackage()] = provider;
    }
    return result;
}
Resource.doNotCapture = true;
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
exports.rootStackResource = undefined;
/**
 * Converts an alias into a URN given a set of default data for the missing
 * values.
 */
function collapseAliasToUrn(alias, defaultName, defaultType, defaultParent) {
    return output_1.output(alias).apply((a) => {
        if (typeof a === "string") {
            return output_1.output(a);
        }
        const name = a.hasOwnProperty("name") ? a.name : defaultName;
        const type = a.hasOwnProperty("type") ? a.type : defaultType;
        const parent = a.hasOwnProperty("parent") ? a.parent : defaultParent;
        const project = a.hasOwnProperty("project") ? a.project : settings_1.getProject();
        const stack = a.hasOwnProperty("stack") ? a.stack : settings_1.getStack();
        if (name === undefined) {
            throw new Error("No valid 'name' passed in for alias.");
        }
        if (type === undefined) {
            throw new Error("No valid 'type' passed in for alias.");
        }
        return createUrn(name, type, parent, project, stack);
    });
}
/**
 * ResourceHook is a named hook that can be registered as a resource hook.
 */
class ResourceHook {
    constructor(name, callback, opts) {
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        this.__pulumiResourceHook = true;
        this.name = name;
        this.callback = callback;
        this.opts = opts;
        this.__registered = resource_1.registerResourceHook(this);
    }
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiResourceHook");
    }
}
exports.ResourceHook = ResourceHook;
/**
 * {@link CustomResource} is a resource whose create, read, update, and delete
 * (CRUD) operations are managed by performing external operations on some
 * physical entity.  The engine understands how to diff and perform partial
 * updates of them, and these CRUD operations are implemented in a dynamically
 * loaded plugin for the defining package.
 */
class CustomResource extends Resource {
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
    constructor(t, name, props, opts = {}, dependency = false, packageRef) {
        if (opts.providers) {
            throw new errors_1.ResourceError("Do not supply 'providers' option to a CustomResource. Did you mean 'provider' instead?", opts.parent);
        }
        super(t, name, true, props, opts, false, dependency, packageRef);
        this.__pulumiCustomResource = true;
    }
    /**
     * Returns true if the given object is a {@link CustomResource}. This is
     * designed to work even when multiple copies of the Pulumi SDK have been
     * loaded into the same process.
     */
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiCustomResource");
    }
}
exports.CustomResource = CustomResource;
CustomResource.doNotCapture = true;
/**
 * {@link ProviderResource} is a resource that implements CRUD operations for
 * other custom resources. These resources are managed similarly to other
 * resources, including the usual diffing and update semantics.
 */
class ProviderResource extends CustomResource {
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
    constructor(pkg, name, props, opts = {}, dependency = false, packageRef) {
        super(`pulumi:providers:${pkg}`, name, props, opts, dependency, packageRef);
        this.pkg = pkg;
    }
    static async register(provider) {
        if (provider === undefined) {
            return undefined;
        }
        if (!provider.__registrationId) {
            const providerURN = await provider.urn.promise();
            const providerID = (await provider.id.promise()) || rpc_1.unknownValue;
            provider.__registrationId = `${providerURN}::${providerID}`;
        }
        return provider.__registrationId;
    }
    /**
     * @internal
     */
    getPackage() {
        return this.pkg;
    }
}
exports.ProviderResource = ProviderResource;
/**
 * {@link ComponentResource} is a resource that aggregates one or more other
 * child resources into a higher level abstraction. The component resource
 * itself is a resource, but does not require custom CRUD operations for
 * provisioning.
 */
class ComponentResource extends Resource {
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
    constructor(type, name, args = {}, opts = {}, remote = false, packageRef) {
        // If the PULUMI_NODEJS_SKIP_COMPONENT_INPUTS environment variable is set,
        // we skip sending the inputs to the engine.
        super(type, name, 
        /*custom:*/ false, process.env.PULUMI_NODEJS_SKIP_COMPONENT_INPUTS ? {} : args, opts, remote, false, packageRef);
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention,no-underscore-dangle,id-blacklist,id-match
        this.__pulumiComponentResource = true;
        /**
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention,no-underscore-dangle,id-blacklist,id-match
        this.__registered = false;
        this.__remote = remote;
        this.__registered = remote || !!opts?.urn;
        this.__data =
            remote || opts?.urn
                ? Promise.resolve({})
                : this.initializeAndRegisterOutputs(args, opts, name, type);
    }
    /**
     * Returns true if the given object is a {@link CustomResource}. This is
     * designed to work even when multiple copies of the Pulumi SDK have been
     * loaded into the same process.
     */
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiComponentResource");
    }
    /**
     * @internal
     */
    async initializeAndRegisterOutputs(args, opts, name, type) {
        const data = await this.initialize(args, opts, name, type);
        this.registerOutputs();
        return data;
    }
    /**
     * Can be overridden by a subclass to asynchronously initialize data for this component
     * automatically when constructed. The data will be available immediately for subclass
     * constructors to use. To access the data use {@link getData}.
     */
    async initialize(args, opts, name, type) {
        return undefined;
    }
    /**
     * Retrieves the data produces by {@link initialize}. The data is
     * immediately available in a derived class's constructor after the
     * `super(...)` call to `ComponentResource`.
     */
    getData() {
        return this.__data;
    }
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
    registerOutputs(outputs) {
        if (this.__registered) {
            return;
        }
        this.__registered = true;
        resource_1.registerResourceOutputs(this, outputs || {});
    }
}
exports.ComponentResource = ComponentResource;
ComponentResource.doNotCapture = true;
ComponentResource.prototype.registerOutputs.doNotCapture = true;
ComponentResource.prototype.initialize.doNotCapture = true;
ComponentResource.prototype.initializeAndRegisterOutputs.doNotCapture = true;
/**
 * @internal
 */
exports.testingOptions = {
    isDryRun: false,
};
function mergeOptions(opts1, opts2) {
    const dest = { ...opts1 };
    const source = { ...opts2 };
    // Ensure provider/providers are all expanded into the `ProviderResource[]` form.
    // This makes merging simple.
    expandProviders(dest);
    expandProviders(source);
    // iterate specifically over the supplied properties in [source].  Note: there may not be an
    // corresponding value in [dest].
    for (const key of Object.keys(source)) {
        const destVal = dest[key];
        const sourceVal = source[key];
        // For 'dependsOn' we might have singleton resources in both options bags. We
        // want to make sure we combine them into a collection.
        if (key === "dependsOn") {
            dest[key] = merge(destVal, sourceVal, /*alwaysCreateArray:*/ true);
            continue;
        }
        if (key === "hooks") {
            continue;
        }
        dest[key] = merge(destVal, sourceVal, /*alwaysCreateArray:*/ false);
    }
    if (dest.hooks || source.hooks) {
        const hookTypes = [
            "beforeCreate",
            "afterCreate",
            "beforeUpdate",
            "afterUpdate",
            "beforeDelete",
            "afterDelete",
        ];
        for (const hookType of hookTypes) {
            const destHooks = dest?.hooks?.[hookType];
            const sourceHooks = source?.hooks?.[hookType];
            if (destHooks || sourceHooks) {
                if (!dest.hooks) {
                    dest.hooks = {};
                }
                dest.hooks[hookType] = merge(destHooks, sourceHooks, /*alwaysCreateArray:*/ false);
            }
        }
    }
    // Now, if we are left with a .providers that is just a single key/value pair, then
    // collapse that down into .provider form.
    normalizeProviders(dest);
    return dest;
}
exports.mergeOptions = mergeOptions;
function isPromiseOrOutput(val) {
    return val instanceof Promise || output_1.Output.isInstance(val);
}
/**
 * @internal
 */
function expandProviders(options) {
    // Convert 'providers' map to array form.
    if (options.providers && !Array.isArray(options.providers)) {
        for (const k in options.providers) {
            if (Object.prototype.hasOwnProperty.call(options.providers, k)) {
                const v = options.providers[k];
                if (k !== v.getPackage()) {
                    const message = `provider resource map where key ${k} doesn't match provider ${v.getPackage()}`;
                    log.warn(message);
                }
            }
        }
        options.providers = utils.values(options.providers);
    }
}
exports.expandProviders = expandProviders;
function normalizeProviders(opts) {
    // If we have 0 providers, delete providers. Otherwise, convert providers into a map.
    const providers = opts.providers;
    if (providers) {
        if (providers.length === 0) {
            opts.providers = undefined;
        }
        else {
            opts.providers = {};
            for (const res of providers) {
                opts.providers[res.getPackage()] = res;
            }
        }
    }
}
/**
 * @internal
 *  This is exported for testing purposes.
 */
function merge(dest, source, alwaysCreateArray) {
    // unwind any top level promise/outputs.
    if (isPromiseOrOutput(dest)) {
        return output_1.output(dest).apply((d) => merge(d, source, alwaysCreateArray));
    }
    if (isPromiseOrOutput(source)) {
        return output_1.output(source).apply((s) => merge(dest, s, alwaysCreateArray));
    }
    // If either are an array, make a new array and merge the values into it.
    // Otherwise, just overwrite the destination with the source value.
    if (alwaysCreateArray || Array.isArray(dest) || Array.isArray(source)) {
        const result = [];
        addToArray(result, dest);
        addToArray(result, source);
        return result;
    }
    return source;
}
exports.merge = merge;
function addToArray(resultArray, value) {
    if (Array.isArray(value)) {
        resultArray.push(...value);
    }
    else if (value !== undefined && value !== null) {
        resultArray.push(value);
    }
}
/**
 * A {@link DependencyResource} is a resource that is used to indicate that an
 * {@link Output} has a dependency on a particular resource. These resources are
 * only created when dealing with remote component resources.
 */
class DependencyResource extends CustomResource {
    constructor(urn) {
        super("", "", {}, {}, true);
        this.urn = new output_1.Output(this, Promise.resolve(urn), Promise.resolve(true), Promise.resolve(false), Promise.resolve([]));
    }
}
exports.DependencyResource = DependencyResource;
/**
 * A {@link DependencyProviderResource} is a resource that is used by the
 * provider SDK as a stand-in for a provider that is only used for its
 * reference. Its only valid properties are its URN and ID.
 */
class DependencyProviderResource extends ProviderResource {
    constructor(ref) {
        const [urn, id] = parseResourceReference(ref);
        const urnParts = urn.split("::");
        const qualifiedType = urnParts[2];
        const type = qualifiedType.split("$").pop();
        // type will be "pulumi:providers:<package>" and we want the last part.
        const typeParts = type.split(":");
        const pkg = typeParts.length > 2 ? typeParts[2] : "";
        super(pkg, "", {}, {}, true);
        this.urn = new output_1.Output(this, Promise.resolve(urn), Promise.resolve(true), Promise.resolve(false), Promise.resolve([]));
        this.id = new output_1.Output(this, Promise.resolve(id), Promise.resolve(true), Promise.resolve(false), Promise.resolve([]));
    }
}
exports.DependencyProviderResource = DependencyProviderResource;
/**
 * Parses the URN and ID out of the provider reference.
 *
 * @internal
 */
function parseResourceReference(ref) {
    const lastSep = ref.lastIndexOf("::");
    if (lastSep === -1) {
        throw new Error(`expected '::' in provider reference ${ref}`);
    }
    const urn = ref.slice(0, lastSep);
    const id = ref.slice(lastSep + 2);
    return [urn, id];
}
exports.parseResourceReference = parseResourceReference;
/**
 * Extracts the package from the type token of the form "pkg:module:member".
 *
 * @internal
 */
function pkgFromType(type) {
    const parts = type.split(":");
    if (parts.length === 3) {
        return parts[0];
    }
    return undefined;
}
exports.pkgFromType = pkgFromType;
/**
 * The Pulumi type assigned to the resource at construction, of the form `package:module:name`.
 */
function resourceType(res) {
    return res.__pulumiType;
}
exports.resourceType = resourceType;
/**
 * The Pulumi name assigned to the resource at construction, i.e. the "name" in its constructor call.
 */
function resourceName(res) {
    if (res.__name === undefined) {
        throw new errors_1.ResourceError("Resource name is not available, this resource instance must have been constructed by an old SDK", res);
    }
    return res.__name;
}
exports.resourceName = resourceName;
//# sourceMappingURL=resource.js.map
"use strict";
// Copyright 2016-2024, Pulumi Corporation.
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
const asset = __importStar(require("../asset"));
const errors_1 = require("../errors");
const log = __importStar(require("../log"));
const output_1 = require("../output");
const resource_1 = require("../resource");
const debuggable_1 = require("./debuggable");
const dependsOn_1 = require("./dependsOn");
const settings_1 = require("./settings");
const state_1 = require("./state");
const semver = __importStar(require("semver"));
// Fix to cover cases such as https://github.com/pulumi/pulumi/issues/17736 where an array or other
// nested object contains an Output, and it's toString method tries to call toString on that output.
function safeToString(v) {
    try {
        return `${v}`;
    }
    catch (err) {
        // We only want to catch our toString errors here. Anything else we should still bubble up to the user.
        if (output_1.OutputToStringError.isInstance(err)) {
            return "Output<T>";
        }
        throw err;
    }
}
/**
 * Mutates the `onto` resource so that it has Promise-valued properties for all
 * the `props` input/output props. *Importantly* all these promises are
 * completely unresolved. This is because we don't want anyone to observe the
 * values of these properties until the rpc call to registerResource actually
 * returns. This is because the registerResource call may actually override
 * input values, and we only want people to see the final value.
 *
 * The result of this call (beyond the stateful changes to `onto`) is the set of
 * {@link Promise} resolvers that will be called post-RPC call.  When the
 * {@link registerResource} RPC call comes back, the values that the engine
 * actualy produced will be used to resolve all the unresolved promised placed
 * on `onto`.
 */
function transferProperties(onto, label, props) {
    const resolvers = {};
    for (const k of Object.keys(props)) {
        // Skip "id" and "urn", since we handle those specially.
        if (k === "id" || k === "urn") {
            continue;
        }
        // Create a property to wrap the value and store it on the resource.
        if (onto.hasOwnProperty(k)) {
            throw new Error(`Property '${k}' is already initialized on target '${label}`);
        }
        let resolveValue;
        let rejectValue;
        let resolveIsKnown;
        let rejectIsKnown;
        let resolveIsSecret;
        let rejectIsSecret;
        let resolveDeps;
        let rejectDeps;
        resolvers[k] = (v, isKnown, isSecret, deps = [], err) => {
            if (err) {
                if (errors_1.isGrpcError(err)) {
                    if (debuggable_1.debugPromiseLeaks) {
                        console.error("info: skipped rejection in transferProperties");
                    }
                    return;
                }
                rejectValue(err);
                rejectIsKnown(err);
                rejectIsSecret(err);
                rejectDeps(err);
            }
            else {
                resolveValue(v);
                resolveIsKnown(isKnown);
                resolveIsSecret(isSecret);
                resolveDeps(deps);
            }
        };
        const propString = output_1.Output.isInstance(props[k]) ? "Output<T>" : safeToString(props[k]);
        onto[k] = new output_1.Output(onto, debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
            resolveValue = resolve;
            rejectValue = reject;
        }), `transferProperty(${label}, ${k}, ${propString})`), debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
            resolveIsKnown = resolve;
            rejectIsKnown = reject;
        }), `transferIsStable(${label}, ${k}, ${propString})`), debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
            resolveIsSecret = resolve;
            rejectIsSecret = reject;
        }), `transferIsSecret(${label}, ${k}, ${propString})`), debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
            resolveDeps = resolve;
            rejectDeps = reject;
        }), `transferDeps(${label}, ${k}, ${propString})`));
    }
    return resolvers;
}
exports.transferProperties = transferProperties;
/**
 * Walks the props object passed in, awaiting all interior promises for
 * properties with keys that match the provided filter, creating a reasonable
 * POJO object that can be remoted over to {@link registerResource}.
 */
async function serializeFilteredProperties(label, props, acceptKey, opts) {
    const propertyToDependentResources = new Map();
    const result = {};
    for (const k of Object.keys(props)) {
        if (acceptKey(k)) {
            // We treat properties with undefined values as if they do not exist.
            const dependentResources = new Set();
            const v = await serializeProperty(`${label}.${k}`, props[k], dependentResources, opts);
            if (v !== undefined) {
                result[k] = v;
                propertyToDependentResources.set(k, dependentResources);
            }
        }
    }
    return [result, propertyToDependentResources];
}
/**
 * Walks the props object passed in, awaiting all interior promises besides
 * those for `id` and `urn`, creating a reasonable POJO object that can be
 * remoted over to {@link registerResource}.
 */
async function serializeResourceProperties(label, props, opts) {
    return serializeFilteredProperties(label, props, (key) => key !== "id" && key !== "urn", opts);
}
exports.serializeResourceProperties = serializeResourceProperties;
/**
 * Walks the props object passed in, awaiting all interior promises, creating a
 * reasonable POJO object that can be remoted over to {@link registerResource}.
 */
async function serializeProperties(label, props, opts) {
    const [result] = await serializeFilteredProperties(label, props, (_) => true, opts);
    return result;
}
exports.serializeProperties = serializeProperties;
/**
 * @internal
 */
async function serializePropertiesReturnDeps(label, props, opts) {
    return serializeFilteredProperties(label, props, (_) => true, opts);
}
exports.serializePropertiesReturnDeps = serializePropertiesReturnDeps;
/**
 * Fetches the raw outputs and deserializes them from a gRPC call result.
 */
function deserializeProperties(outputsStruct, keepUnknowns) {
    const props = {};
    const outputs = outputsStruct.toJavaScript();
    for (const k of Object.keys(outputs)) {
        // We treat properties with undefined values as if they do not exist.
        if (outputs[k] !== undefined) {
            props[k] = deserializeProperty(outputs[k], keepUnknowns);
        }
    }
    return props;
}
exports.deserializeProperties = deserializeProperties;
/**
 * Takes as input a gRPC serialized `proto.google.protobuf.Struct` and resolves
 * all of the resource's matching properties to the values inside.
 *
 * NOTE: it is imperative that the properties in `allProps` were produced by
 * `deserializeProperties` in order for output properties to work correctly
 * w.r.t. knowns/unknowns: this function assumes that any undefined value in
 * `allProps`represents an unknown value that was returned by an engine
 * operation.
 */
function resolveProperties(res, resolvers, t, name, allProps, deps, err, keepUnknowns) {
    // If there is an error, just reject everything.
    if (err) {
        for (const k of Object.keys(resolvers)) {
            const resolve = resolvers[k];
            resolve(undefined, true, false, [], err);
        }
        return;
    }
    // Now go ahead and resolve all properties present in the inputs and outputs set.
    for (const k of Object.keys(allProps)) {
        // Skip "id" and "urn", since we handle those specially.
        if (k === "id" || k === "urn") {
            continue;
        }
        // Otherwise, unmarshal the value, and store it on the resource object.
        const resolve = resolvers[k];
        if (resolve === undefined) {
            // engine returned a property that was not in our initial property-map.  This can happen
            // for outputs that were registered through direct calls to 'registerOutputs'. We do
            // *not* want to do anything with these returned properties.  First, the component
            // resources that were calling 'registerOutputs' will have already assigned these fields
            // directly on them themselves.  Second, if we were to try to assign here we would have
            // an incredibly bad race condition for two reasons:
            //
            //  1. This call to 'resolveProperties' happens asynchronously at some point far after
            //     the resource was constructed.  So the user will have been able to observe the
            //     initial value up until we get to this point.
            //
            //  2. The component resource will have often assigned a value of some arbitrary type
            //     (say, a 'string').  If we overwrite this with an `Output<string>` we'll be changing
            //     the type at some non-deterministic point in the future.
            continue;
        }
        // If this value is a secret, unwrap its inner value.
        let value = allProps[k];
        const isSecret = isRpcSecret(value);
        value = unwrapRpcSecret(value);
        try {
            // If the value the engine handed back is or contains an unknown value, the resolver will mark its value as
            // unknown automatically, so we just pass true for isKnown here. Note that unknown values will only be
            // present during previews (i.e. isDryRun() will be true).
            resolve(value, /*isKnown*/ true, isSecret, deps[k]);
        }
        catch (resolveError) {
            throw new Error(`Unable to set property '${k}' on resource '${name}' [${t}]; error: ${debuggable_1.errorString(resolveError)}`);
        }
    }
    // `allProps` may not have contained a value for every resolver: for example, optional outputs may not be present.
    // We will resolve all of these values as `undefined`, and will mark the value as known if we are not running a
    // preview.  For updates when the update of the resource was either skipped or failed we'll mark them as `unknown`.
    for (const k of Object.keys(resolvers)) {
        if (!allProps.hasOwnProperty(k)) {
            const resolve = resolvers[k];
            if (!settings_1.isDryRun && keepUnknowns) {
                resolve(output_1.unknown, true, false);
            }
            else {
                resolve(undefined, !settings_1.isDryRun() && !keepUnknowns, false);
            }
        }
    }
}
exports.resolveProperties = resolveProperties;
/**
 * Unknown values are encoded as a distinguished string value.
 */
exports.unknownValue = "04da6b54-80e4-46f7-96ec-b56ff0331ba9";
/**
 * {@link specialSigKey} is sometimes used to encode type identity inside of a
 * map.
 *
 * @see sdk/go/common/resource/properties.go.
 */
exports.specialSigKey = "4dabf18193072939515e22adb298388d";
/**
 * {@link specialAssetSig} is a randomly assigned hash used to identify assets
 * in maps.
 *
 * @see sdk/go/common/resource/asset.go.
 */
exports.specialAssetSig = "c44067f5952c0a294b673a41bacd8c17";
/**
 * {@link specialArchiveSig} is a randomly assigned hash used to identify
 * archives in maps.
 *
 * @see sdk/go/common/resource/asset.go.
 */
exports.specialArchiveSig = "0def7320c3a5731c473e5ecbe6d01bc7";
/**
 * {@link specialSecretSig} is a randomly assigned hash used to identify secrets
 * in maps.
 *
 * @see sdk/go/common/resource/properties.go.
 */
exports.specialSecretSig = "1b47061264138c4ac30d75fd1eb44270";
/**
 * {@link specialResourceSig} is a randomly assigned hash used to identify
 * resources in maps.
 *
 * @see sdk/go/common/resource/properties.go.
 */
exports.specialResourceSig = "5cf8f73096256a8f31e491e813e4eb8e";
/**
 * {@link specialOutputValueSig} is a randomly assigned hash used to identify
 * outputs in maps.
 *
 * @see sdk/go/common/resource/properties.go.
 */
exports.specialOutputValueSig = "d0e6a833031e9bbcd3f4e8bde6ca49a4";
/** @internal */
function serializeSecretValue(value) {
    return {
        [exports.specialSigKey]: exports.specialSecretSig,
        // coerce 'undefined' to 'null' as required by the protobuf system.
        value: value === undefined ? null : value,
    };
}
exports.serializeSecretValue = serializeSecretValue;
/**
 * Serializes properties deeply.  This understands how to wait on any unresolved
 * promises, as appropriate, in addition to translating certain "special" values
 * so that they are ready to go on the wire.
 */
async function serializeProperty(ctx, prop, dependentResources, opts) {
    // if prop is a function type just return undefined, we don't want to try and keep these values around.
    if (typeof prop === "function") {
        if (settings_1.excessiveDebugOutput) {
            log.debug(`Serialize property [${ctx}]: function=${prop}`);
        }
        return undefined;
    }
    if (prop === undefined ||
        prop === null ||
        typeof prop === "boolean" ||
        typeof prop === "number" ||
        typeof prop === "string") {
        if (settings_1.excessiveDebugOutput) {
            log.debug(`Serialize property [${ctx}]: primitive=${prop}`);
        }
        return prop;
    }
    if (asset.Asset.isInstance(prop) || asset.Archive.isInstance(prop)) {
        // Serializing an asset or archive requires the use of a magical signature key, since otherwise it would look
        // like any old weakly typed object/map when received by the other side of the RPC boundary.
        const obj = {
            [exports.specialSigKey]: asset.Asset.isInstance(prop) ? exports.specialAssetSig : exports.specialArchiveSig,
        };
        return await serializeAllKeys(prop, obj, { keepOutputValues: false });
    }
    if (prop instanceof Promise) {
        // For a promise input, await the property and then serialize the result.
        if (settings_1.excessiveDebugOutput) {
            log.debug(`Serialize property [${ctx}]: Promise<T>`);
        }
        const subctx = `Promise<${ctx}>`;
        return serializeProperty(subctx, await debuggable_1.debuggablePromise(prop, `serializeProperty.await(${subctx})`), dependentResources, opts);
    }
    if (output_1.Output.isInstance(prop)) {
        if (settings_1.excessiveDebugOutput) {
            log.debug(`Serialize property [${ctx}]: Output<T>`);
        }
        // handle serializing both old-style outputs (with sync resources) and new-style outputs
        // (with async resources).
        const propResources = await output_1.getAllResources(prop);
        for (const resource of propResources) {
            if (dependentResources) {
                dependentResources.add(resource);
            }
        }
        // When serializing an Output, we will either serialize it as its resolved value or the "unknown value"
        // sentinel. We will do the former for all outputs created directly by user code (such outputs always
        // resolve isKnown to true) and for any resource outputs that were resolved with known values.
        const isKnown = await prop.isKnown;
        // You might think that doing an explict `=== true` here is not needed, but it is for a subtle reason. If the
        // output we are serializing is a proxy itself, and it comes from a version of the SDK that did not have the
        // `isSecret` member on `OutputImpl` then the call to `prop.isSecret` here will return an Output itself,
        // which will wrap undefined, if it were to be resolved (since `Output` has no member named .isSecret).
        // so we must compare to the literal true instead of just doing await prop.isSecret.
        const isSecret = (await prop.isSecret) === true;
        const promiseDeps = new Set();
        const value = await serializeProperty(`${ctx}.id`, prop.promise(), promiseDeps, {
            keepOutputValues: false,
        });
        for (const resource of promiseDeps) {
            propResources.add(resource);
            if (dependentResources) {
                dependentResources.add(resource);
            }
        }
        if (opts?.keepOutputValues && state_1.getStore().supportsOutputValues) {
            const urnDeps = new Set();
            for (const resource of propResources) {
                await serializeProperty(`${ctx} dependency`, resource.urn, urnDeps, {
                    keepOutputValues: false,
                });
            }
            for (const resource of urnDeps) {
                propResources.add(resource);
                if (dependentResources) {
                    dependentResources.add(resource);
                }
            }
            const dependencies = await dependsOn_1.getAllTransitivelyReferencedResourceURNs(propResources, new Set());
            const obj = {
                [exports.specialSigKey]: exports.specialOutputValueSig,
            };
            if (isKnown) {
                // coerce 'undefined' to 'null' as required by the protobuf system.
                obj["value"] = value === undefined ? null : value;
            }
            if (isSecret) {
                obj["secret"] = isSecret;
            }
            if (dependencies.size > 0) {
                obj["dependencies"] = Array.from(dependencies);
            }
            return obj;
        }
        if (!isKnown) {
            return exports.unknownValue;
        }
        if (isSecret && state_1.getStore().supportsSecrets) {
            return serializeSecretValue(value);
        }
        return value;
    }
    if (output_1.isUnknown(prop)) {
        return exports.unknownValue;
    }
    if (resource_1.CustomResource.isInstance(prop)) {
        if (settings_1.excessiveDebugOutput) {
            log.debug(`Serialize property [${ctx}]: custom resource urn`);
        }
        if (opts?.excludeResourceReferencesFromDependencies && state_1.getStore().supportsResourceReferences) {
            // If excluding resource references from dependencies and the monitor supports resource
            // references, we don't want to track this dependency, so we set `dependentResources`
            // to undefined so when serializing the `id` and `urn` the resource won't be included
            // in the caller's `dependentResources`.
            dependentResources = undefined;
        }
        else {
            // Otherwise, track the dependency.
            if (dependentResources) {
                dependentResources.add(prop);
            }
        }
        const id = await serializeProperty(`${ctx}.id`, prop.id, dependentResources, {
            keepOutputValues: false,
        });
        if (state_1.getStore().supportsResourceReferences) {
            // If we are keeping resources, emit a stronly typed wrapper over the URN
            const urn = await serializeProperty(`${ctx}.urn`, prop.urn, dependentResources, {
                keepOutputValues: false,
            });
            return {
                [exports.specialSigKey]: exports.specialResourceSig,
                urn: urn,
                id: id,
            };
        }
        // Else, return the id for backward compatibility.
        return id;
    }
    if (resource_1.ComponentResource.isInstance(prop)) {
        // Component resources often can contain cycles in them.  For example, an awsinfra
        // SecurityGroupRule can point a the awsinfra SecurityGroup, which in turn can point back to
        // its rules through its `egressRules` and `ingressRules` properties.  If serializing out
        // the `SecurityGroup` resource ends up trying to serialize out those properties, a deadlock
        // will happen, due to waiting on the child, which is waiting on the parent.
        //
        // Practically, there is no need to actually serialize out a component.  It doesn't represent
        // a real resource, nor does it have normal properties that need to be tracked for differences
        // (since changes to its properties don't represent changes to resources in the real world).
        //
        // So, to avoid these problems, while allowing a flexible and simple programming model, we
        // just serialize out the component as its urn.  This allows the component to be identified
        // and tracked in a reasonable manner, while not causing us to compute or embed information
        // about it that is not needed, and which can lead to deadlocks.
        if (settings_1.excessiveDebugOutput) {
            log.debug(`Serialize property [${ctx}]: component resource urn`);
        }
        if (state_1.getStore().supportsResourceReferences) {
            if (opts?.excludeResourceReferencesFromDependencies) {
                // If excluding resource references from dependencies and the monitor supports resource
                // references, we don't want to track this dependency, so we set `dependentResources`
                // to undefined so when serializing the `urn` the resource won't be included in the
                // caller's `dependentResources`.
                dependentResources = undefined;
            }
            // If we are keeping resources, emit a strongly typed wrapper over the URN
            const urn = await serializeProperty(`${ctx}.urn`, prop.urn, dependentResources, {
                keepOutputValues: false,
            });
            return {
                [exports.specialSigKey]: exports.specialResourceSig,
                urn: urn,
            };
        }
        // Else, return the urn for backward compatibility.
        return serializeProperty(`${ctx}.urn`, prop.urn, dependentResources, {
            keepOutputValues: false,
        });
    }
    if (prop instanceof Array) {
        const result = [];
        for (let i = 0; i < prop.length; i++) {
            if (settings_1.excessiveDebugOutput) {
                log.debug(`Serialize property [${ctx}]: array[${i}] element`);
            }
            // When serializing arrays, we serialize any undefined values as `null`. This matches JSON semantics.
            const elem = await serializeProperty(`${ctx}[${i}]`, prop[i], dependentResources, opts);
            result.push(elem === undefined ? null : elem);
        }
        return result;
    }
    return await serializeAllKeys(prop, {}, opts);
    async function serializeAllKeys(innerProp, obj, innerOpts) {
        for (const k of Object.keys(innerProp)) {
            if (settings_1.excessiveDebugOutput) {
                log.debug(`Serialize property [${ctx}]: object.${k}`);
            }
            // When serializing an object, we omit any keys with undefined values. This matches JSON semantics.
            const v = await serializeProperty(`${ctx}.${k}`, innerProp[k], dependentResources, innerOpts);
            if (v !== undefined) {
                obj[k] = v;
            }
        }
        return obj;
    }
}
exports.serializeProperty = serializeProperty;
/**
 * Returns true if the given object is a wrapped secret value (i.e. it's an
 * object with the special key set).
 */
function isRpcSecret(obj) {
    return obj && obj[exports.specialSigKey] === exports.specialSecretSig;
}
exports.isRpcSecret = isRpcSecret;
/**
 * Returns the underlying value for a secret, or the value itself if it was not
 * a secret.
 */
function unwrapRpcSecret(obj) {
    if (!isRpcSecret(obj)) {
        return obj;
    }
    return obj.value;
}
exports.unwrapRpcSecret = unwrapRpcSecret;
function isPrimitive(value) {
    return (value === null ||
        value === undefined ||
        typeof value === "boolean" ||
        typeof value === "number" ||
        typeof value === "string");
}
/** @internal */
function containsUnknownValues(value) {
    if (value === exports.unknownValue) {
        return true;
    }
    else if (isPrimitive(value)) {
        return false;
    }
    else if (value instanceof Array) {
        return value.some(containsUnknownValues);
    }
    else {
        const map = value;
        return Object.keys(map).some((key) => containsUnknownValues(map[key]));
    }
}
exports.containsUnknownValues = containsUnknownValues;
/** @internal */
function unwrapSecretValues(value) {
    if (isPrimitive(value)) {
        return [value, false];
    }
    else if (isRpcSecret(value)) {
        return [unwrapRpcSecret(value), true];
    }
    else if (value instanceof Array) {
        let hadSecret = false;
        const result = [];
        for (const elem of value) {
            const [unwrapped, isSecret] = unwrapSecretValues(elem);
            hadSecret = hadSecret || isSecret;
            result.push(unwrapped);
        }
        return [result, hadSecret];
    }
    else {
        let hadSecret = false;
        const result = {};
        const map = value;
        for (const key of Object.keys(map)) {
            const [unwrapped, isSecret] = unwrapSecretValues(map[key]);
            hadSecret = hadSecret || isSecret;
            result[key] = unwrapped;
        }
        return [result, hadSecret];
    }
}
exports.unwrapSecretValues = unwrapSecretValues;
/**
 * Unpacks some special types, reversing the process undertaken by
 * {@link serializeProperty}.
 */
function deserializeProperty(prop, keepUnknowns) {
    if (prop === undefined) {
        throw new Error("unexpected undefined property value during deserialization");
    }
    else if (prop === exports.unknownValue) {
        return settings_1.isDryRun() || keepUnknowns ? output_1.unknown : undefined;
    }
    else if (prop === null || typeof prop === "boolean" || typeof prop === "number" || typeof prop === "string") {
        return prop;
    }
    else if (prop instanceof Array) {
        // We can just deserialize all the elements of the underlying array and return it.
        // However, we want to push secretness up to the top level (since we can't set sub-properties to secret)
        // values since they are not typed as Output<T>.
        let hadSecret = false;
        const elems = [];
        for (const e of prop) {
            prop = deserializeProperty(e, keepUnknowns);
            hadSecret = hadSecret || isRpcSecret(prop);
            elems.push(unwrapRpcSecret(prop));
        }
        if (hadSecret) {
            return {
                [exports.specialSigKey]: exports.specialSecretSig,
                value: elems,
            };
        }
        return elems;
    }
    else {
        // We need to recognize assets and archives specially, so we can produce the right runtime objects.
        const sig = prop[exports.specialSigKey];
        if (sig) {
            switch (sig) {
                case exports.specialAssetSig:
                    if (prop["path"]) {
                        return new asset.FileAsset(prop["path"]);
                    }
                    else if (prop["text"]) {
                        return new asset.StringAsset(prop["text"]);
                    }
                    else if (prop["uri"]) {
                        return new asset.RemoteAsset(prop["uri"]);
                    }
                    else {
                        throw new Error("Invalid asset encountered when unmarshaling resource property");
                    }
                case exports.specialArchiveSig:
                    if (prop["assets"]) {
                        const assets = {};
                        for (const name of Object.keys(prop["assets"])) {
                            const a = deserializeProperty(prop["assets"][name], keepUnknowns);
                            if (!asset.Asset.isInstance(a) && !asset.Archive.isInstance(a)) {
                                throw new Error("Expected an AssetArchive's assets to be unmarshaled Asset or Archive objects");
                            }
                            assets[name] = a;
                        }
                        return new asset.AssetArchive(assets);
                    }
                    else if (prop["path"]) {
                        return new asset.FileArchive(prop["path"]);
                    }
                    else if (prop["uri"]) {
                        return new asset.RemoteArchive(prop["uri"]);
                    }
                    else {
                        throw new Error("Invalid archive encountered when unmarshaling resource property");
                    }
                case exports.specialSecretSig:
                    return {
                        [exports.specialSigKey]: exports.specialSecretSig,
                        value: deserializeProperty(prop["value"], keepUnknowns),
                    };
                case exports.specialResourceSig:
                    // Deserialize the resource into a live Resource reference
                    const urn = prop["urn"];
                    const version = prop["packageVersion"];
                    const urnParts = urn.split("::");
                    const qualifiedType = urnParts[2];
                    const urnName = urnParts[3];
                    const type = qualifiedType.split("$").pop();
                    const typeParts = type.split(":");
                    const pkgName = typeParts[0];
                    const modName = typeParts.length > 1 ? typeParts[1] : "";
                    const typName = typeParts.length > 2 ? typeParts[2] : "";
                    const isProvider = pkgName === "pulumi" && modName === "providers";
                    if (isProvider) {
                        const resourcePackage = getResourcePackage(typName, version);
                        if (resourcePackage) {
                            return resourcePackage.constructProvider(urnName, type, urn);
                        }
                    }
                    else {
                        const resourceModule = getResourceModule(pkgName, modName, version);
                        if (resourceModule) {
                            return resourceModule.construct(urnName, type, urn);
                        }
                    }
                    // If we've made it here, deserialize the reference as either a URN or an ID (if present).
                    if (prop["id"]) {
                        const id = prop["id"];
                        return deserializeProperty(id === "" ? exports.unknownValue : id, keepUnknowns);
                    }
                    return urn;
                case exports.specialOutputValueSig:
                    let value = prop["value"];
                    const isKnown = value !== undefined;
                    if (isKnown) {
                        value = deserializeProperty(value, keepUnknowns);
                    }
                    const isSecret = prop["secret"] === true;
                    const dependencies = prop["dependencies"];
                    const resources = Array.isArray(dependencies)
                        ? dependencies.map((d) => new resource_1.DependencyResource(d))
                        : [];
                    return new output_1.Output(resources, Promise.resolve(value), Promise.resolve(isKnown), Promise.resolve(isSecret), Promise.resolve([]));
                default:
                    throw new Error(`Unrecognized signature '${sig}' when unmarshaling resource property`);
            }
        }
        // If there isn't a signature, it's not a special type, and we can simply return the object as a map.
        // However, we want to push secretness up to the top level (since we can't set sub-properties to secret)
        // values since they are not typed as Output<T>.
        const obj = {};
        let hadSecrets = false;
        for (const k of Object.keys(prop)) {
            const o = deserializeProperty(prop[k], keepUnknowns);
            hadSecrets = hadSecrets || isRpcSecret(o);
            obj[k] = unwrapRpcSecret(o);
        }
        if (hadSecrets) {
            return {
                [exports.specialSigKey]: exports.specialSecretSig,
                value: obj,
            };
        }
        return obj;
    }
}
exports.deserializeProperty = deserializeProperty;
/**
 * Silences any unhandled promise rejections that occur due to gRPC errors. The
 * input promise may still be rejected.
 */
function suppressUnhandledGrpcRejections(p) {
    p.catch((err) => {
        if (!errors_1.isGrpcError(err)) {
            throw err;
        }
    });
    return p;
}
exports.suppressUnhandledGrpcRejections = suppressUnhandledGrpcRejections;
function sameVersion(a, b) {
    // We treat undefined as a wildcard, so it always equals every other version.
    return a === undefined || b === undefined || semver.eq(a, b);
}
function checkVersion(want, have) {
    if (want === undefined || have === undefined) {
        return true;
    }
    return have.major === want.major && have.minor >= want.minor && have.patch >= want.patch;
}
/**
 * @internal
 */
function register(source, registrationType, key, item) {
    let items = source.get(key);
    if (items) {
        for (const existing of items) {
            if (sameVersion(existing.version, item.version)) {
                // It is possible for the same version of the same provider SDK to be loaded multiple times in Node.js.
                // In this case, we might legitimately get multiple registrations of the same resource.  It should not
                // matter which we use, so we can just skip re-registering.  De-serialized resources will always be
                // instances of classes from the first registered package.
                if (settings_1.excessiveDebugOutput) {
                    log.debug(`skip re-registering already registered ${registrationType} ${key}@${item.version}.`);
                }
                return false;
            }
        }
    }
    else {
        items = [];
        source.set(key, items);
    }
    if (settings_1.excessiveDebugOutput) {
        log.debug(`registering ${registrationType} ${key}@${item.version}`);
    }
    items.push(item);
    return true;
}
exports.register = register;
/**
 * @internal
 */
function getRegistration(source, key, version) {
    const ver = version ? new semver.SemVer(version) : undefined;
    let bestMatch = undefined;
    let bestMatchVersion = undefined;
    for (const existing of source.get(key) ?? []) {
        const existingVersion = existing.version !== undefined ? new semver.SemVer(existing.version) : undefined;
        if (!checkVersion(ver, existingVersion)) {
            continue;
        }
        if (!bestMatch || (existingVersion && bestMatchVersion && semver.gt(existingVersion, bestMatchVersion))) {
            bestMatch = existing;
            bestMatchVersion = existingVersion;
        }
    }
    return bestMatch;
}
exports.getRegistration = getRegistration;
/**
 * @internal
 *  Used only for testing purposes.
 */
function _resetResourcePackages() {
    const resourcePackages = state_1.getResourcePackages();
    resourcePackages.clear();
}
exports._resetResourcePackages = _resetResourcePackages;
/**
 * Registers a resource package that will be used to construct providers for any
 * URNs matching the package name and version that are deserialized by the
 * current instance of the Pulumi JavaScript SDK.
 */
function registerResourcePackage(pkg, resourcePackage) {
    const resourcePackages = state_1.getResourcePackages();
    register(resourcePackages, "package", pkg, resourcePackage);
}
exports.registerResourcePackage = registerResourcePackage;
function getResourcePackage(pkg, version) {
    const resourcePackages = state_1.getResourcePackages();
    return getRegistration(resourcePackages, pkg, version);
}
exports.getResourcePackage = getResourcePackage;
function moduleKey(pkg, mod) {
    return `${pkg}:${mod}`;
}
/**
 * @internal
 *  Used only for testing purposes.
 */
function _resetResourceModules() {
    const resourceModules = state_1.getResourceModules();
    resourceModules.clear();
}
exports._resetResourceModules = _resetResourceModules;
/**
 * Registers a resource module that will be used to construct resources for any
 * URNs matching the module name and version that are deserialized by the
 * current instance of the Pulumi JavaScript SDK.
 */
function registerResourceModule(pkg, mod, module) {
    const key = moduleKey(pkg, mod);
    const resourceModules = state_1.getResourceModules();
    register(resourceModules, "module", key, module);
}
exports.registerResourceModule = registerResourceModule;
function getResourceModule(pkg, mod, version) {
    const key = moduleKey(pkg, mod);
    const resourceModules = state_1.getResourceModules();
    return getRegistration(resourceModules, key, version);
}
exports.getResourceModule = getResourceModule;
//# sourceMappingURL=rpc.js.map
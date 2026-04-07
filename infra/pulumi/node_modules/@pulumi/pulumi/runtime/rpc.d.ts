import { Input, Inputs } from "../output";
import { ProviderResource, Resource } from "../resource";
import * as gstruct from "google-protobuf/google/protobuf/struct_pb";
export declare type OutputResolvers = Record<string, (value: any, isStable: boolean, isSecret: boolean, deps?: Resource[], err?: Error) => void>;
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
export declare function transferProperties(onto: Resource, label: string, props: Inputs): OutputResolvers;
/**
 * Controls the serialization of RPC structures.
 */
export interface SerializationOptions {
    /**
     * True if we are keeping output values. If the monitor does not support
     * output values, they will not be kept, even when this is set to true.
     */
    keepOutputValues?: boolean;
    /**
     * True if we should exclude resource references from the set of dependencies
     * identified during serialization. This is useful for remote components where
     * we want propertyDependencies to be empty for a property that only contains
     * resource references. If the monitor does not support resource references,
     * this will have no effect.
     */
    excludeResourceReferencesFromDependencies?: boolean;
}
/**
 * Walks the props object passed in, awaiting all interior promises besides
 * those for `id` and `urn`, creating a reasonable POJO object that can be
 * remoted over to {@link registerResource}.
 */
export declare function serializeResourceProperties(label: string, props: Inputs, opts?: SerializationOptions): Promise<[Record<string, any>, Map<string, Set<Resource>>]>;
/**
 * Walks the props object passed in, awaiting all interior promises, creating a
 * reasonable POJO object that can be remoted over to {@link registerResource}.
 */
export declare function serializeProperties(label: string, props: Inputs, opts?: SerializationOptions): Promise<Record<string, any>>;
/**
 * Fetches the raw outputs and deserializes them from a gRPC call result.
 */
export declare function deserializeProperties(outputsStruct: gstruct.Struct, keepUnknowns?: boolean): Inputs;
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
export declare function resolveProperties(res: Resource, resolvers: Record<string, (v: any, isKnown: boolean, isSecret: boolean, deps?: Resource[], err?: Error) => void>, t: string, name: string, allProps: any, deps: Record<string, Resource[]>, err?: Error, keepUnknowns?: boolean): void;
/**
 * Unknown values are encoded as a distinguished string value.
 */
export declare const unknownValue = "04da6b54-80e4-46f7-96ec-b56ff0331ba9";
/**
 * {@link specialSigKey} is sometimes used to encode type identity inside of a
 * map.
 *
 * @see sdk/go/common/resource/properties.go.
 */
export declare const specialSigKey = "4dabf18193072939515e22adb298388d";
/**
 * {@link specialAssetSig} is a randomly assigned hash used to identify assets
 * in maps.
 *
 * @see sdk/go/common/resource/asset.go.
 */
export declare const specialAssetSig = "c44067f5952c0a294b673a41bacd8c17";
/**
 * {@link specialArchiveSig} is a randomly assigned hash used to identify
 * archives in maps.
 *
 * @see sdk/go/common/resource/asset.go.
 */
export declare const specialArchiveSig = "0def7320c3a5731c473e5ecbe6d01bc7";
/**
 * {@link specialSecretSig} is a randomly assigned hash used to identify secrets
 * in maps.
 *
 * @see sdk/go/common/resource/properties.go.
 */
export declare const specialSecretSig = "1b47061264138c4ac30d75fd1eb44270";
/**
 * {@link specialResourceSig} is a randomly assigned hash used to identify
 * resources in maps.
 *
 * @see sdk/go/common/resource/properties.go.
 */
export declare const specialResourceSig = "5cf8f73096256a8f31e491e813e4eb8e";
/**
 * {@link specialOutputValueSig} is a randomly assigned hash used to identify
 * outputs in maps.
 *
 * @see sdk/go/common/resource/properties.go.
 */
export declare const specialOutputValueSig = "d0e6a833031e9bbcd3f4e8bde6ca49a4";
/**
 * Serializes properties deeply.  This understands how to wait on any unresolved
 * promises, as appropriate, in addition to translating certain "special" values
 * so that they are ready to go on the wire.
 */
export declare function serializeProperty(ctx: string, prop: Input<any>, dependentResources?: Set<Resource>, opts?: SerializationOptions): Promise<any>;
/**
 * Returns true if the given object is a wrapped secret value (i.e. it's an
 * object with the special key set).
 */
export declare function isRpcSecret(obj: any): boolean;
/**
 * Returns the underlying value for a secret, or the value itself if it was not
 * a secret.
 */
export declare function unwrapRpcSecret(obj: any): any;
/**
 * Unpacks some special types, reversing the process undertaken by
 * {@link serializeProperty}.
 */
export declare function deserializeProperty(prop: any, keepUnknowns?: boolean): any;
/**
 * Silences any unhandled promise rejections that occur due to gRPC errors. The
 * input promise may still be rejected.
 */
export declare function suppressUnhandledGrpcRejections<T>(p: Promise<T>): Promise<T>;
/**
 * A {@link ResourcePackage} is a type that understands how to construct
 * resource providers given a name, type, args, and URN.
 */
export interface ResourcePackage {
    readonly version?: string;
    constructProvider(name: string, type: string, urn: string): ProviderResource;
}
/**
 * Registers a resource package that will be used to construct providers for any
 * URNs matching the package name and version that are deserialized by the
 * current instance of the Pulumi JavaScript SDK.
 */
export declare function registerResourcePackage(pkg: string, resourcePackage: ResourcePackage): void;
export declare function getResourcePackage(pkg: string, version: string | undefined): ResourcePackage | undefined;
/**
 * A {@link ResourceModule} is a type that understands how to construct
 * resources given a name, type, args, and URN.
 */
export interface ResourceModule {
    readonly version?: string;
    construct(name: string, type: string, urn: string): Resource;
}
/**
 * Registers a resource module that will be used to construct resources for any
 * URNs matching the module name and version that are deserialized by the
 * current instance of the Pulumi JavaScript SDK.
 */
export declare function registerResourceModule(pkg: string, mod: string, module: ResourceModule): void;
export declare function getResourceModule(pkg: string, mod: string, version: string | undefined): ResourceModule | undefined;

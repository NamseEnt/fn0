import { Resource } from "./resource";
/**
 * Creates an Output<T> whose value can be later resolved from another Output<T> instance.
 */
export declare function deferredOutput<T>(): [Output<T>, (source: Output<T>) => void];
/**
 * @Internal
 */
export declare function getAllResources<T>(op: OutputInstance<T>): Promise<Set<Resource>>;
/**
 * {@link output} takes any {@link Input} value and converts it into an
 * {@link Output}, deeply unwrapping nested {@link Input} values as necessary.
 *
 * The expected way to use this function is like so:
 *
 * ```ts
 *      var transformed = pulumi.output(someVal).apply(unwrapped => {
 *          // Do whatever you want now.  'unwrapped' will contain no outputs/promises inside
 *          // here, so you can easily do whatever sort of transformation is most convenient.
 *      });
 *
 *      // the result can be passed to another Resource.  The dependency information will be
 *      // properly maintained.
 *      var someResource = new SomeResource(name, { data: transformed ... });
 * ```
 */
export declare function output<T>(val: Input<T>): Output<Unwrap<T>>;
export declare function output<T>(val: Input<T> | undefined): Output<Unwrap<T | undefined>>;
/**
 * {@link secret} behaves the same as {@link output} except the returned output
 * is marked as containing sensitive data.
 */
export declare function secret<T>(val: Input<T>): Output<Unwrap<T>>;
export declare function secret<T>(val: Input<T> | undefined): Output<Unwrap<T | undefined>>;
/**
 * {@link unsecret} behaves the same as {@link output} except the returned
 * output takes the existing output and unwraps the secret.
 */
export declare function unsecret<T>(val: Output<T>): Output<T>;
/**
 * {@link isSecret} returns `true` if and only if the provided {@link Output} is
 * a secret.
 */
export declare function isSecret<T>(val: Output<T>): Promise<boolean>;
/**
 * Allows for multiple {@link Output} objects to be combined into a single
 * {@link Output} object.  The single {@link Output} will depend on the union of
 * {@link Resources} that the individual dependencies depend on.
 *
 * This can be used in the following manner:
 *
 * ```ts
 * var d1: Output<string>;
 * var d2: Output<number>;
 *
 * var d3: Output<ResultType> = Output.all([d1, d2]).apply(([s, n]) => ...);
 * ```
 *
 * In this example, taking a dependency on `d3` means a resource will depend on
 * all the resources of `d1` and `d2`.
 */
export declare function all<T>(val: Record<string, Input<T>>): Output<Record<string, Unwrap<T>>>;
export declare function all<T1, T2, T3, T4, T5, T6, T7, T8>(values: [Input<T1>, Input<T2>, Input<T3>, Input<T4>, Input<T5>, Input<T6>, Input<T7>, Input<T8>]): Output<[Unwrap<T1>, Unwrap<T2>, Unwrap<T3>, Unwrap<T4>, Unwrap<T5>, Unwrap<T6>, Unwrap<T7>, Unwrap<T8>]>;
export declare function all<T1, T2, T3, T4, T5, T6, T7>(values: [Input<T1>, Input<T2>, Input<T3>, Input<T4>, Input<T5>, Input<T6>, Input<T7>]): Output<[Unwrap<T1>, Unwrap<T2>, Unwrap<T3>, Unwrap<T4>, Unwrap<T5>, Unwrap<T6>, Unwrap<T7>]>;
export declare function all<T1, T2, T3, T4, T5, T6>(values: [Input<T1>, Input<T2>, Input<T3>, Input<T4>, Input<T5>, Input<T6>]): Output<[Unwrap<T1>, Unwrap<T2>, Unwrap<T3>, Unwrap<T4>, Unwrap<T5>, Unwrap<T6>]>;
export declare function all<T1, T2, T3, T4, T5>(values: [Input<T1>, Input<T2>, Input<T3>, Input<T4>, Input<T5>]): Output<[Unwrap<T1>, Unwrap<T2>, Unwrap<T3>, Unwrap<T4>, Unwrap<T5>]>;
export declare function all<T1, T2, T3, T4>(values: [Input<T1>, Input<T2>, Input<T3>, Input<T4>]): Output<[Unwrap<T1>, Unwrap<T2>, Unwrap<T3>, Unwrap<T4>]>;
export declare function all<T1, T2, T3>(values: [Input<T1>, Input<T2>, Input<T3>]): Output<[Unwrap<T1>, Unwrap<T2>, Unwrap<T3>]>;
export declare function all<T1, T2>(values: [Input<T1>, Input<T2>]): Output<[Unwrap<T1>, Unwrap<T2>]>;
export declare function all<T>(ds: Input<T>[]): Output<Unwrap<T>[]>;
/**
 * Unknown represents a value that is unknown. These values correspond to
 * unknown property values received from the Pulumi engine as part of the result
 * of a resource registration (see `runtime/rpc.ts`). User code is not typically
 * exposed to these values: any {@link Output} that contains an {@link Unknown}
 * will itself be unknown, so any user callbacks passed to `apply` will not be
 * run. Internal callers of `apply` can request that they are run even with
 * unknown values; the output proxy takes advantage of this to allow proxied
 * property accesses to return known values even if other properties of the
 * containing object are unknown.
 */
declare class Unknown {
    /**
     * Returns true if the given object is an {@link Unknown}. This is designed
     * to work even when multiple copies of the Pulumi SDK have been loaded into
     * the same process.
     */
    static isInstance(obj: any): obj is Unknown;
}
/**
 * {@link unknown} is the singleton {@link Unknown} value.
 *
 */
export declare const unknown: Unknown;
/**
 * Returns true if the given value is unknown.
 */
export declare function isUnknown(val: any): boolean;
/**
 * Returns true if the given value is or contains unknown values.
 */
export declare function containsUnknowns(value: any): boolean;
/**
 * {@link Input} is a property input for a resource. It may be a promptly
 * available `T`, a promise for one, or the {@link Output} from a existing
 * {@link Resource}.
 */
export declare type Input<T> = T | Promise<T> | OutputInstance<T>;
/**
 * {@link Inputs} is a map of property name to property input, one for each
 * resource property value.
 */
export declare type Inputs = Record<string, Input<any>>;
/**
 * The {@link Unwrap} type allows us to express the operation of taking a type,
 * with potentially deeply nested {@link Promise}s and {@link Output}s and to
 * then get that same type with all the promises and outputs replaced with their
 * wrapped type.  Note that this unwrapping is "deep". So if you had:
 *
 *      `type X = { A: Promise<{ B: Output<{ c: Input<boolean> }> }> }`
 *
 * Then `Unwrap<X>` would be equivalent to:
 *
 *      `...    = { A: { B: { C: boolean } } }`
 *
 * Unwrapping sees through promises, outputs, arrays and objects.
 *
 * Note: due to TypeScript limitations there are some things that cannot be
 * expressed. Specifically, if you had a `Promise<Output<T>>` then the {@link
 * Unwrap} type would not be able to undo both of those wraps. In practice that
 * should be OK. Values in an object graph should not wrap outputs in promises.
 * Instead, any code that needs to work Outputs and also be async should either
 * create the output with the promise (which will collapse into just an output).
 * Or, it should start with an output and call `apply` on it, passing in an
 * `async` function. This will also collapse and just produce an output.
 *
 * In other words, this should not be used as the shape of an object: `{ a:
 * Promise<Output<...>> }`. It should always either be `{ a: Promise<NonOutput>
 * }` or just `{ a: Output<...> }`.
 */
export declare type Unwrap<T> = T extends Promise<infer U1> ? UnwrapSimple<U1> : T extends OutputInstance<infer U2> ? UnwrapSimple<U2> : UnwrapSimple<T>;
declare type primitive = Function | string | number | boolean | undefined | null;
/**
 * Handles encountering basic types when unwrapping.
 */
export declare type UnwrapSimple<T> = T extends primitive ? T : T extends Resource ? T : T extends [any, ...any[]] ? UnwrappedObject<T> : T extends Array<infer U> ? UnwrappedArray<U> : T extends object ? UnwrappedObject<T> : never;
export declare type UnwrappedArray<T> = Array<Unwrap<T>>;
export declare type UnwrappedObject<T> = {
    [P in keyof T]: Unwrap<T[P]>;
};
/**
 * Instance side of the {@link Output} type. Exposes the deployment-time and
 * run-time mechanisms for working with the underlying value of an {@link Output}.
 */
export interface OutputInstance<T> {
    /**
     * Transforms the data of the output with the provided `func`. The result
     * remains an {@link Output} so that dependent resources can be properly
     * tracked.
     *
     * `func` should not be used to create resources unless necessary as `func` may not be run during some program executions.
     *
     * `func` can return other {@link Output}s. This can be handy if you have an
     * `Output<SomeVal>` and you want to get a transitive dependency of it,
     * i.e.
     *
     * ```ts
     * var d1: Output<SomeVal>;
     * var d2 = d1.apply(v => v.x.y.OtherOutput); // getting an output off of 'v'.
     * ```
     *
     * In this example, taking a dependency on `d2` means a resource will depend
     * on all the resources of `d1`.  It will *also* depend on the resources of
     * `v.x.y.OtherDep`.
     *
     * Importantly, the resources that `d2` feels like it will depend on are the
     * same resources as `d1`. If you need have multiple outputs and a single
     * output is needed that combines both set of resources, then `pulumi.all`
     * should be used instead.
     *
     * This function will be called during the execution of a `pulumi up` or
     * `pulumi preview` operation, but it will not run when the values of the
     * output are unknown. It is not available for functions that end up
     * executing in the cloud during runtime. To get the value of the Output
     * during cloud runtime execution, use `get()`.
     */
    apply<U>(func: (t: T) => Promise<U>): Output<U>;
    apply<U>(func: (t: T) => OutputInstance<U>): Output<U>;
    apply<U>(func: (t: T) => U): Output<U>;
    /**
     * Retrieves the underlying value of this dependency.
     *
     * This function is only callable in code that runs in the cloud
     * post-deployment. At this point all {@link Output} values will be known
     * and can be safely retrieved. During Pulumi deployment or preview
     * execution this must not be called (and will throw). This is because doing
     * so would allow output values to flow into resources while losing the data
     * that would allow the dependency graph to be changed.
     */
    get(): T;
}
/**
 * Static side of the {@link Output} type. Can be used to create outputs as well
 * as test arbitrary values to see if they are {@link Output}s.
 */
export interface OutputConstructor {
    create<T>(val: Input<T>): Output<Unwrap<T>>;
    create<T>(val: Input<T> | undefined): Output<Unwrap<T | undefined>>;
    isInstance<T>(obj: any): obj is Output<T>;
}
/**
 * {@link Output} helps encode the relationship between {@link Resource}s in a
 * Pulumi application. Specifically, an {@link Output} holds onto a piece of
 * data and the resource it was generated from. An output value can then be
 * provided when constructing new resources, allowing that new resource to know
 * both the value as well as the resource the value came from.  This allows for
 * a precise resource dependency graph to be created, which properly tracks the
 * relationship between resources.
 *
 * An output is used in a Pulumi program differently depending on if the
 * application is executing at "deployment time" (i.e. when actually running the
 * `pulumi` executable), or at "run time" (i.e. a piece of code running in some
 * cloud).
 *
 * At "deployment time", the correct way to work with the underlying value is to
 * call {@link Output.apply}. This allows the value to be accessed and
 * manipulated, while still resulting in an output that is keeping track of
 * {@link Resource}s appropriately. At deployment time the underlying value may
 * or may not exist (for example, if a preview is being performed).  In this
 * case, the `func` callback will not be executed, and calling `.apply` will
 * immediately return an output that points to the `undefined` value. During a
 * normal update though, the `func` callbacks should always be executed.
 *
 * At "run time", the correct way to work with the underlying value is to simply
 * call {@link Output.get} which will be promptly return the entire value.  This
 * will be a simple JavaScript object that can be manipulated as necessary.
 *
 * To ease with using outputs at deployment time, Pulumi will "lift" simple data
 * properties of an underlying value to the output itself.  For example:
 *
 * ```ts
 *      const o: Output<{ name: string, age: number, orders: Order[] }> = ...;
 *      const name : Output<string> = o.name;
 *      const age  : Output<number> = o.age;
 *      const first: Output<Order>  = o.orders[0];
 * ```
 *
 * Instead of having to write:
 *
 * ```ts
 *      const o: Output<{ name: string, age: number, orders: Order[] }> = ...;
 *      const name : Output<string> = o.apply(v => v.name);
 *      const age  : Output<number> = o.apply(v => v.age);
 *      const first: Output<Order> = o.apply(v => v.orders[0]);
 * ```
 */
export declare type Output<T> = OutputInstance<T> & Lifted<T>;
export declare const Output: OutputConstructor;
/**
 * The {@link Lifted} type allows us to express the operation of taking a type,
 * with potentially deeply nested objects and arrays and to then get a type with
 * the same properties, except whose property types are now {@link Output}s of the
 * original property type.
 *
 * For example:
 *
 *
 *      `type X = { A: string, B: { c: boolean } }`
 *
 * Then `Lifted<X>` would be equivalent to:
 *
 *      `...    = { A: Output<string>, B: Output<{ C: Output<boolean> }> }`
 *
 * {@link Lifted} is somewhat the opposite of {@link Unwrap}. Its primary
 * purpose is to allow an instance of `Output<SomeType>` to provide simple
 * access to the properties of `SomeType` directly on the instance itself
 * (instead of haveing to use {@link Output.apply}).
 *
 * This lifting only happens through simple objects and arrays. Functions, for
 * example, are not lifted. So you cannot do:
 *
 * ```ts
 *      const o: Output<string> = ...;
 *      const c: Output<number> = o.charCodeAt(0);
 * ```
 *
 * Instead, you still need to write;
 *
 * ```ts
 *      const o: Output<string> = ...;
 *      const c: Output<number> = o.apply(v => v.charCodeAt(0));
 * ```
 */
export declare type Lifted<T> = T extends string ? LiftedObject<String, NonFunctionPropertyNames<String>> : T extends Array<infer U> ? LiftedArray<U> : T extends object ? LiftedObject<T, NonFunctionPropertyNames<T>> : {};
declare type NonFunctionPropertyNames<T> = {
    [K in keyof T]: T[K] extends Function ? never : K;
}[keyof T];
export declare type LiftedObject<T, K extends keyof T> = {
    [P in K]: IsStrictlyAny<T[P]> extends true ? Output<any> : T[P] extends OutputInstance<infer T1> ? Output<T1> : T[P] extends Promise<infer T2> ? Output<T2> : Output<T[P]>;
};
declare type IsStrictlyAny<T> = (T extends never ? true : false) extends false ? false : true;
export declare type LiftedArray<T> = {
    /**
     * Gets the length of the array. This is a number one higher than the highest element defined
     * in an array.
     */
    readonly length: Output<number>;
    readonly [n: number]: Output<T>;
};
/**
 * {@link concat} takes a sequence of {@link Input}s, stringifies each one, and
 * concatenates all values into one final string.  Individual inputs can be any
 * sort of input value: they can be promises, outputs, or just plain JavaScript
 * values. Use this function like so:
 *
 * ```ts
 *      // 'server' and 'loadBalancer' are both resources that expose [Output] properties.
 *      let val: Output<string> = pulumi.concat("http://", server.hostname, ":", loadBalancer.port);
 * ```
 *
 */
export declare function concat(...params: Input<any>[]): Output<string>;
/**
 * {@link interpolate} is similar to {@link concat} but is designed to be used
 * as a tagged template expression, e.g.:
 *
 * ```ts
 *      // 'server' and 'loadBalancer' are both resources that expose [Output] properties.
 *      let val: Output<string> = pulumi.interpolate `http://${server.hostname}:${loadBalancer.port}`
 * ```
 *
 * As with {@link concat}, the placeholders between `${}` can be any
 * {@link Input}s: promises, outputs, or just plain JavaScript values.
 */
export declare function interpolate(literals: TemplateStringsArray, ...placeholders: Input<any>[]): Output<string>;
/**
 * {@link jsonStringify} uses {@link JSON.stringify} to serialize the given
 * {@link Input} value into a JSON string.
 */
export declare function jsonStringify(obj: Input<any>, replacer?: (this: any, key: string, value: any) => any | (number | string)[], space?: string | number): Output<string>;
/**
 * {@link jsonParse} Uses {@link JSON.parse} to deserialize the given {@link
 * Input} JSON string into a value.
 */
export declare function jsonParse(text: Input<string>, reviver?: (this: any, key: string, value: any) => any): Output<any>;
export {};

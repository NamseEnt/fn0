import { InvokeOptions, InvokeOutputOptions } from "../invoke";
import { Inputs, Output } from "../output";
import { Resource } from "../resource";
/**
 * Dynamically invokes the function `tok`, which is offered by a provider
 * plugin. `invoke` behaves differently in the case that options contains
 * `{async:true}` or not.
 *
 * In the case where `{async:true}` is present in the options bag:
 *
 * 1. the result of `invoke` will be a Promise resolved to the result value of
 *    the provider plugin.
 *
 * 2. the `props` inputs can be a bag of computed values (including, `T`s,
 *   `Promise<T>`s, `Output<T>`s etc.).
 *
 * In the case where `{ async:true }` is not present in the options bag:
 *
 * 1. the result of `invoke` will be a Promise resolved to the result value of
 *    the provider call. However, that Promise will *also* have the respective
 *    values of the Provider result exposed directly on it as properties.
 *
 * 2. The inputs must be a bag of simple values, and the result is the result
 *    that the Provider produced.
 *
 * Simple values are:
 *
 *  1. `undefined`, `null`, string, number or boolean values.
 *  2. arrays of simple values.
 *  3. objects containing only simple values.
 *
 * Importantly, simple values do *not* include:
 *
 *  1. `Promise`s
 *  2. `Output`s
 *  3. `Asset`s or `Archive`s
 *  4. `Resource`s.
 *
 * All of these contain async values that would prevent `invoke from being able
 * to operate synchronously.
 */
export declare function invoke(tok: string, props: Inputs, opts?: InvokeOptions, packageRef?: Promise<string | undefined>): Promise<any>;
/**
 * Similar to the plain `invoke` but returns the response as an output, maintaining
 * secrets of the response, if any.
 */
export declare function invokeOutput<T>(tok: string, props: Inputs, opts?: InvokeOutputOptions, packageRef?: Promise<string | undefined>): Output<T>;
export declare function invokeSingle(tok: string, props: Inputs, opts?: InvokeOptions, packageRef?: Promise<string | undefined>): Promise<any>;
/**
 * Similar to the plain `invokeSingle` but returns the response as an output, maintaining
 * secrets of the response, if any.
 */
export declare function invokeSingleOutput<T>(tok: string, props: Inputs, opts?: InvokeOptions, packageRef?: Promise<string | undefined>): Output<T>;
/**
 * Calls a method `tok` offered by a provider plugin resource.
 */
export declare function call<T>(tok: string, props: Inputs, res?: Resource, packageRef?: Promise<string | undefined>): Output<T>;
/**
 * Calls a method `tok` offered by a provider plugin resource, but returns a single value.
 *
 * This method expects the result of `callAsync` to be a map containing a single value,
 * which it unwraps.
 */
export declare function callSingle<T>(tok: string, props: Inputs, res?: Resource, packageRef?: Promise<string | undefined>): Output<T>;

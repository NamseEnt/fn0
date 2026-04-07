import { OutputInstance } from "../index";
import { Resource } from "../resource";
/**
 * {@link ResolvedResource} is a {@link Resource} with all fields containing
 * {@link Output} values fully resolved.
 */
export declare type ResolvedResource<T extends Resource> = Omit<Resolved<T>, "urn" | "getProvider">;
export declare type Resolved<T> = T extends Promise<infer U1> ? ResolvedSimple<U1> : T extends OutputInstance<infer U2> ? ResolvedSimple<U2> : ResolvedSimple<T>;
declare type primitive = string | number | boolean | undefined | null;
declare type ResolvedSimple<T> = T extends primitive ? T : T extends Array<infer U> ? ResolvedArray<U> : T extends Function ? never : T extends object ? ResolvedObject<T> : never;
declare type ResolvedArray<T> = Array<Resolved<T>>;
declare type ResolvedObject<T> = ModifyOptionalProperties<{
    [P in keyof T]: Resolved<T[P]>;
}>;
declare type RequiredKeys<T> = {
    [P in keyof T]: undefined extends T[P] ? never : P;
}[keyof T];
declare type OptionalKeys<T> = {
    [P in keyof T]: undefined extends T[P] ? P : never;
}[keyof T];
declare type ModifyOptionalProperties<T> = {
    [P in RequiredKeys<T>]: T[P];
} & {
    [P in OptionalKeys<T>]?: T[P];
};
export {};

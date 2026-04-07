/// <reference types="node" />
import { AsyncLocalStorage } from "async_hooks";
import { Stack } from "./stack";
declare global {
    var globalStore: Store;
    var asyncLocalStorage: AsyncLocalStorage<Store>;
    var stackResource: Stack | undefined;
}

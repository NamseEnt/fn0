export declare function getBoundFunction(func: Function): Promise<{
    targetFunctionText: string;
    boundThisValue: any;
    boundArgsValues: any[];
}>;

export declare const Runtime: {
    readonly Dotnet6: "dotnet6";
    readonly Dotnet8: "dotnet8";
    readonly Java11: "java11";
    readonly Java17: "java17";
    readonly Java21: "java21";
    readonly Java8AL2: "java8.al2";
    readonly NodeJS18dX: "nodejs18.x";
    readonly NodeJS20dX: "nodejs20.x";
    readonly NodeJS22dX: "nodejs22.x";
    readonly CustomAL2: "provided.al2";
    readonly CustomAL2023: "provided.al2023";
    readonly Python3d10: "python3.10";
    readonly Python3d11: "python3.11";
    readonly Python3d12: "python3.12";
    readonly Python3d13: "python3.13";
    readonly Python3d9: "python3.9";
    readonly Ruby3d2: "ruby3.2";
    readonly Ruby3d3: "ruby3.3";
    readonly Ruby3d4: "ruby3.4";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Dotnet5d0: "dotnet5.0";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Dotnet7: "dotnet7";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly DotnetCore2d1: "dotnetcore2.1";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly DotnetCore3d1: "dotnetcore3.1";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Go1dx: "go1.x";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Java8: "java8";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly NodeJS10dX: "nodejs10.x";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly NodeJS12dX: "nodejs12.x";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly NodeJS14dX: "nodejs14.x";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly NodeJS16dX: "nodejs16.x";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Custom: "provided";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Python2d7: "python2.7";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Python3d6: "python3.6";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Python3d7: "python3.7";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Python3d8: "python3.8";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Ruby2d5: "ruby2.5";
    /**
     * @deprecated This runtime is now deprecated
     */
    readonly Ruby2d7: "ruby2.7";
};
/**
 * See https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtimes.html
 */
export type Runtime = (typeof Runtime)[keyof typeof Runtime];

"use strict";
// Copyright 2016-2018, Pulumi Corporation.
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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs = __importStar(require("fs"));
const fdir_1 = require("fdir");
const normalize_package_data_1 = __importDefault(require("normalize-package-data"));
const arborist = __importStar(require("@npmcli/arborist"));
const upath = __importStar(require("upath"));
const __1 = require("../..");
const asset = __importStar(require("../../asset"));
const errors_1 = require("../../errors");
async function computeCodePaths(optionsOrExtraIncludePaths, extraIncludePackages, extraExcludePackages) {
    let options;
    if (Array.isArray(optionsOrExtraIncludePaths)) {
        __1.log.warn("'function computeCodePaths(string[])' is deprecated. Use the [computeCodePaths] overload that takes a [CodePathOptions] instead.");
        options = {
            extraIncludePaths: optionsOrExtraIncludePaths,
            extraIncludePackages,
            extraExcludePackages,
        };
    }
    else {
        options = optionsOrExtraIncludePaths || {};
    }
    return computeCodePathsWorker(options);
}
exports.computeCodePaths = computeCodePaths;
async function computeCodePathsWorker(options) {
    // Construct the set of paths to include in the archive for upload.
    // Find folders for all packages requested by the user.  Note: all paths in this should
    // be normalized.
    const normalizedPathSet = await allFoldersForPackages(new Set(options.extraIncludePackages || []), new Set(options.extraExcludePackages || []), options.logResource);
    // Add all paths explicitly requested by the user
    const extraIncludePaths = options.extraIncludePaths || [];
    for (const path of extraIncludePaths) {
        normalizedPathSet.add(upath.normalize(path));
    }
    const codePaths = new Map();
    // For each of the required paths, add the corresponding FileArchive or FileAsset to the
    // AssetMap.
    for (const normalizedPath of normalizedPathSet) {
        // Don't include a path if there is another path higher up that will include this one.
        if (isSubsumedByHigherPath(normalizedPath, normalizedPathSet)) {
            continue;
        }
        // The Asset model does not support a consistent way to embed a file-or-directory into an
        // `AssetArchive`, so we stat the path to figure out which it is and use the appropriate
        // Asset constructor.
        const stats = fs.statSync(normalizedPath);
        if (stats.isDirectory()) {
            codePaths.set(normalizedPath, new asset.FileArchive(normalizedPath));
        }
        else {
            codePaths.set(normalizedPath, new asset.FileAsset(normalizedPath));
        }
    }
    return codePaths;
}
function isSubsumedByHigherPath(normalizedPath, normalizedPathSet) {
    for (const otherNormalizedPath of normalizedPathSet) {
        if (normalizedPath.length > otherNormalizedPath.length && normalizedPath.startsWith(otherNormalizedPath)) {
            // Have to make sure we're actually a sub-directory of that other path.  For example,
            // if we have:  node_modules/mime-types, that's not subsumed by node_modules/mime
            const nextChar = normalizedPath.charAt(otherNormalizedPath.length);
            return nextChar === "/";
        }
    }
    return false;
}
/**
 * Searches for and returns the first directory path starting from a given
 * directory that contains the given file to find. Recursively searches up the
 * directory tree until it finds the file or returns `null` when it can't find
 * anything.
 * */
function searchUp(currentDir, fileToFind) {
    if (fs.existsSync(upath.join(currentDir, fileToFind))) {
        return currentDir;
    }
    const parentDir = upath.resolve(currentDir, "..");
    if (currentDir === parentDir) {
        return null;
    }
    return searchUp(parentDir, fileToFind);
}
/**
 * Detects if we are in a Yarn/NPM workspace setup, and returns the root of the
 * workspace. If we are not in a workspace setup, it returns `null`.
 *
 * @internal
 */
async function findWorkspaceRoot(startingPath) {
    const stat = fs.statSync(startingPath);
    if (!stat.isDirectory()) {
        startingPath = upath.dirname(startingPath);
    }
    const packageJSONDir = searchUp(startingPath, "package.json");
    if (packageJSONDir === null) {
        return null;
    }
    // We start at the location of the first `package.json` we find.
    let currentDir = packageJSONDir;
    let nextDir = upath.dirname(currentDir);
    while (currentDir !== nextDir) {
        const p = upath.join(currentDir, "package.json");
        if (!fs.existsSync(p)) {
            currentDir = nextDir;
            nextDir = upath.dirname(currentDir);
            continue;
        }
        const workspaces = parseWorkspaces(p);
        for (const workspace of workspaces) {
            const globber = new fdir_1.fdir().withBasePath().glob(upath.join(currentDir, workspace, "package.json"));
            const files = await globber.crawl(currentDir).withPromise();
            const normalized = upath.normalizeTrim(upath.join(packageJSONDir, "package.json"));
            if (files.map((f) => upath.normalizeTrim(f)).includes(normalized)) {
                return currentDir;
            }
        }
        currentDir = nextDir;
        nextDir = upath.dirname(currentDir);
    }
    return null;
}
exports.findWorkspaceRoot = findWorkspaceRoot;
function parseWorkspaces(packageJsonPath) {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
    if (packageJson.workspaces && Array.isArray(packageJson.workspaces)) {
        return packageJson.workspaces;
    }
    if (packageJson.workspaces?.packages && Array.isArray(packageJson.workspaces.packages)) {
        return packageJson.workspaces.packages;
    }
    return [];
}
/**
 * Computes the set of package folders that are transitively required by the root
 * `dependencies` node in the client's `package.json` file.
 */
async function allFoldersForPackages(includedPackages, excludedPackages, logResource) {
    // the working directory is the directory containing the package.json file
    let workingDir = searchUp(".", "package.json");
    if (workingDir === null) {
        // we couldn't find a directory containing package.json
        // searching up from the current directory
        throw new errors_1.ResourceError("Failed to find package.json.", logResource);
    }
    workingDir = upath.resolve(workingDir);
    // This is the core starting point of the algorithm.  We read the
    // package.json information for this project, and then we start by walking
    // the .dependencies node in that package.  Importantly, we do not look at
    // things like .devDependencies or or .peerDependencies.  These are not
    // what are considered part of the final runtime configuration of the app
    // and should not be uploaded.
    const referencedPackages = new Set(includedPackages);
    const packageJSON = computeDependenciesDirectlyFromPackageFile(upath.join(workingDir, "package.json"), logResource);
    if (packageJSON.dependencies) {
        for (const depName of Object.keys(packageJSON.dependencies)) {
            referencedPackages.add(depName);
        }
    }
    // Find the workspace root, fallback to current working directory if we are not in a workspaces setup.
    let workspaceRoot = (await findWorkspaceRoot(workingDir)) || workingDir;
    // Ensure workingDir is a relative path so we get relative paths in the
    // output. If we have absolute paths, AWS lambda might not find the
    // dependencies.
    workspaceRoot = upath.relative(upath.resolve("."), workspaceRoot);
    // Read package tree from the workspace root to ensure we can find all
    // packages in the workspace.  We then call addPackageAndDependenciesToSet
    // to recursively add all the dependencies of the referenced packages.
    const arb = new arborist.Arborist({ path: workspaceRoot });
    const root = await arb.loadActual();
    // package.json files can contain circularities.  For example es6-iterator depends
    // on es5-ext, which depends on es6-iterator, which depends on es5-ext:
    // https://github.com/medikoo/es6-iterator/blob/0eac672d3f4bb3ccc986bbd5b7ffc718a0822b74/package.json#L20
    // https://github.com/medikoo/es5-ext/blob/792c9051e5ad9d7671dd4e3957eee075107e9e43/package.json#L29
    //
    // So keep track of the paths we've looked and don't recurse if we hit something again.
    const seenPaths = new Set();
    const normalizedPackagePaths = new Set();
    for (const pkg of referencedPackages) {
        addPackageAndDependenciesToSet(root, pkg, seenPaths, normalizedPackagePaths, excludedPackages);
    }
    return normalizedPackagePaths;
}
function computeDependenciesDirectlyFromPackageFile(path, logResource) {
    // read the package.json file in directly.  if any of these fail an error will be thrown
    // and bubbled back out to user.
    const contents = readFile();
    const data = parse();
    // 'normalize-package-data' can throw if 'version' isn't a valid string.  We don't care about
    // 'version' so just delete it.
    // https://github.com/npm/normalize-package-data/blob/df8ea05b8cd38531e8b70ac7906f420344f55bab/lib/fixer.js#L191
    data.version = undefined;
    // 'normalize-package-data' can throw if 'name' isn't a valid string.  We don't care about
    // 'name' so just delete it.
    // https://github.com/npm/normalize-package-data/blob/df8ea05b8cd38531e8b70ac7906f420344f55bab/lib/fixer.js#L211
    data.name = undefined;
    normalize_package_data_1.default(data);
    return data;
    function readFile() {
        try {
            return fs.readFileSync(path);
        }
        catch (err) {
            throw new errors_1.ResourceError(`Error reading file '${path}' when computing package dependencies. ${err}`, logResource);
        }
    }
    function parse() {
        try {
            return JSON.parse(contents.toString());
        }
        catch (err) {
            throw new errors_1.ResourceError(`Error parsing file '${path}' when computing package dependencies. ${err}`, logResource);
        }
    }
}
/**
 * Adds all required dependencies for the requested package name from the given
 * root package into the set. It will recurse into all dependencies of the
 * package.
 */
function addPackageAndDependenciesToSet(root, pkg, seenPaths, normalizedPackagePaths, excludedPackages) {
    // Don't process this packages if it was in the set the user wants to exclude.
    if (excludedPackages.has(pkg)) {
        return;
    }
    const child = findDependency(root, pkg);
    if (!child) {
        console.warn(`Could not include required dependency '${pkg}' in '${upath.resolve(root.path)}'.`);
        return;
    }
    // Don't process a child path if we've already encountered it.
    const normalizedPath = upath.relative(upath.resolve("."), upath.resolve(child.path));
    if (seenPaths.has(normalizedPath)) {
        return;
    }
    seenPaths.add(normalizedPath);
    if (child.package.pulumi) {
        // This was a pulumi deployment-time package.  Check if it had a:
        //
        //    `pulumi: { runtimeDependencies: ... }`
        //
        // section.  In this case, we don't want to add this specific package, but we do want to
        // include all the runtime dependencies it says are necessary.
        recurse(child.package.pulumi.runtimeDependencies);
    }
    else if (pkg.startsWith("@pulumi")) {
        // exclude it if it's an @pulumi package.  These packages are intended for deployment
        // time only and will only bloat up the serialized lambda package.  Note: this code can
        // be removed once all pulumi packages add a "pulumi" section to their package.json.
        return;
    }
    else {
        // Normal package.  Add the normalized path to it, and all transitively add all of its
        // dependencies.
        normalizedPackagePaths.add(normalizedPath);
        recurse(child.package.dependencies);
    }
    return;
    function recurse(dependencies) {
        if (dependencies) {
            for (const dep of Object.keys(dependencies)) {
                let next;
                if (child?.isLink) {
                    next = child.target;
                }
                else {
                    next = child;
                }
                addPackageAndDependenciesToSet(next, dep, seenPaths, normalizedPackagePaths, excludedPackages);
            }
        }
    }
}
/**
 * Searches the package tree starting at a root node (possibly a child) for a
 * match for the given name. It is assumed that the tree was correctly
 * constructed such that dependencies are resolved to compatible versions in the
 * closest available match starting at the provided root and walking up to the
 * head of the tree.
 */
function findDependency(root, name) {
    while (root) {
        for (const [childName, child] of root.children) {
            if (childName === name) {
                return child;
            }
        }
        if (root.parent) {
            root = root.parent;
        }
        else if (root.root !== root) {
            root = root.root; // jump up to the workspace root
        }
        else {
            break;
        }
    }
    return undefined;
}
//# sourceMappingURL=codePaths.js.map
deno_core::extension!(
    onejs,
    deps = [
        deno_console,
        deno_webidl,
        deno_url,
        deno_web,
        deno_fetch,
        deno_net,
        deno_crypto,
        deno_http
    ],
    esm_entry_point = "ext:onejs/bootstrap.js",
    esm = [
        dir "js",
        "bootstrap.js",
    ]
);

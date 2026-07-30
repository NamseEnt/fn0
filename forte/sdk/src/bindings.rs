wit_bindgen::generate!({
    inline: "
        package forte:sdk;
        world imports-only {
            import wasi:cli/types@0.3.0-rc-2026-03-15;
            import wasi:cli/stdin@0.3.0-rc-2026-03-15;
            import wasi:clocks/types@0.3.0-rc-2026-03-15;
            import wasi:clocks/monotonic-clock@0.3.0-rc-2026-03-15;
            import wasi:clocks/system-clock@0.3.0-rc-2026-03-15;
            import wasi:clocks/timezone@0.3.0-rc-2026-03-15;
            import wasi:random/random@0.3.0-rc-2026-03-15;
            import wasi:random/insecure@0.3.0-rc-2026-03-15;
            import wasi:random/insecure-seed@0.3.0-rc-2026-03-15;
            import wasi:http/types@0.3.0-rc-2026-03-15;
            import wasi:http/client@0.3.0-rc-2026-03-15;
        }
    ",
    path: "wit",
    world: "imports-only",
    generate_all,
    features: ["clocks-timezone"],
});

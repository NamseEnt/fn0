export default {
  async fetch() {
    return Response.json({
      promising: typeof WebAssembly.promising,
      Suspending: typeof WebAssembly.Suspending,
      wasmKeys: Object.getOwnPropertyNames(WebAssembly),
    });
  },
};

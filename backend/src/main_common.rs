pub fn setup_deno_runtime() -> anyhow::Result<()> {
    // https://github.com/denoland/deno/blob/main/cli/main.rs#L477
    #[cfg(feature = "deno_core")]
    let unrecognized_v8_flags = deno_core::v8_set_flags(vec![
        "--stack-size=1024".to_string(),
        // TODO(bartlomieju): I think this can be removed as it's handled by `deno_core`
        // and its settings.
        // deno_ast removes TypeScript `assert` keywords, so this flag only affects JavaScript
        // TODO(petamoriken): Need to check TypeScript `assert` keywords in deno_ast
        "--no-harmony-import-assertions".to_string(),
    ])
    .into_iter()
    .skip(1)
    .collect::<Vec<_>>();

    #[cfg(feature = "deno_core")]
    if !unrecognized_v8_flags.is_empty() {
        println!("Unrecognized V8 flags: {:?}", unrecognized_v8_flags);
    }

    #[cfg(feature = "deno_core")]
    deno_core::JsRuntime::init_platform(None, false);
}

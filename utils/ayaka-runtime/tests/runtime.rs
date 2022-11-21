#[generic_tests::define(attrs(tokio::test))]
mod runtime_tests {
    use ayaka_plugin::RawModule;
    use ayaka_plugin_wasmer::WasmerModule;
    use ayaka_plugin_wasmi::WasmiModule;
    use ayaka_plugin_wasmtime::WasmtimeModule;
    use ayaka_runtime::{plugin::HostRuntime, script::*, *};

    async fn with_ctx<M: RawModule>(f: impl FnOnce(&mut VarTable<M>)) {
        let runtime = HostRuntime::load(
            "../../examples/plugins",
            env!("CARGO_MANIFEST_DIR"),
            &["random"],
        )
        .await
        .unwrap();
        let mut locals = VarMap::default();
        let mut ctx = VarTable::new(&runtime, &mut locals);
        f(&mut ctx);
    }

    #[tokio::test]
    async fn vars<M: RawModule>() {
        with_ctx::<M>(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        "
                        a = 0;
                        a += 1;
                        a += a;
                        a
                    "
                    )
                    .ok()
                    .call(ctx),
                RawValue::Num(2)
            );

            assert_eq!(
                ProgramParser::new().parse("a").ok().call(ctx),
                RawValue::Unit
            );

            assert_eq!(
                ProgramParser::new()
                    .parse(
                        "
                        $a = 0;
                        $a += 1;
                        $a += a;
                        $a
                    "
                    )
                    .ok()
                    .call(ctx),
                RawValue::Num(1)
            );

            assert_eq!(
                ProgramParser::new().parse("$a").ok().call(ctx),
                RawValue::Num(1)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn if_test<M: RawModule>() {
        with_ctx::<M>(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        r##"
                        if(1 + 1 + 4 + 5 + 1 + 4 == 16, "sodayo", ~)
                    "##
                    )
                    .ok()
                    .call(ctx)
                    .get_num(),
                6
            );
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        r##"
                        if(true, "sodayo")
                    "##
                    )
                    .ok()
                    .call(ctx)
                    .get_str(),
                "sodayo"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn random<M: RawModule>() {
        with_ctx::<M>(|ctx| {
            assert!((0..10).contains(
                &ProgramParser::new()
                    .parse(
                        r##"
                        random.rnd(10)
                    "##
                    )
                    .ok()
                    .call(ctx)
                    .get_num()
            ))
        })
        .await;
    }

    #[instantiate_tests(<WasmiModule>)]
    mod inst_wasmi {}
    #[instantiate_tests(<WasmtimeModule>)]
    mod inst_wasmtime {}
    #[instantiate_tests(<WasmerModule>)]
    mod inst_wasmer {}
}

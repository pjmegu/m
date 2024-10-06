use criterion::{criterion_group, criterion_main, Criterion};

fn bench_call_host(c: &mut Criterion) {
    c.bench_function("call_host", |b| {
        b.iter(|| {
            let univ = bugic::Universe::new();
            let mut host = bugic::host_plug::HostPlugin::new();
            host.host_func::<bugic::param::BitcodeTag, bugic::param::BitcodeTag, _, _, _>(
                "hello".to_string(),
                |param: (String,)| format!("Hello, {}!", param.0),
            );

            let pref = univ
                .add_host_plugin("Hello".to_string(), host)
                .expect("Failed to add plugin");

            let result = pref
                .call::<bugic::param::BitcodeTag, bugic::param::BitcodeTag, _, String>(
                    "hello".to_string(),
                    ("World".to_string(),),
                )
                .expect("Failed to call plugin");

            assert_eq!(result, "Hello, World!".to_string());
        });
    });
}

criterion_group!(benches, bench_call_host);
criterion_main!(benches);

#![cfg(test)]

use anyhow::*;
use bugi::*;

#[test]
fn host_call() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(name,): (String,), _| format!("Hello, {}!", name));

    let pref = univ.add_plugin("host_test", host)?;

    let res = pref.call::<RmpTag, String>("test", ("world".to_string(),))?;

    assert_eq!(res, "Hello, world!");
    Ok(())
}

#[test]
fn cache_test() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(name,): (String,), _| format!("Hello, {}!", name));

    let pref = univ.add_plugin("host_test", host)?;

    let cacher = Cacher::new();

    let res = pref.call_cache::<RmpTag, String>("test", ("world".to_string(),), &cacher)?;

    assert_eq!(res, "Hello, world!");

    let res = pref.call_cache::<RmpTag, String>("test", ("world".to_string(),), &cacher)?;

    assert_eq!(res, "Hello, world!");

    Ok(())
}

#[test]
fn override_fn_test() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(a, b): (i32, i32), ploxy| {
        ploxy
            .call_univ::<RmpTag, i32>("test2", "called", (a, b))
            .unwrap()
    });
    let pref = univ.add_plugin("test", host)?;

    let mut over = Overrider::new();
    over.add::<RmpTag, _, _>("test2", "called", |(a, b): (i32, i32)| a * b);
    let res = over.wrap_call::<RmpTag, i32>(&pref, "test", (5, 10))?;

    assert_eq!(res, 50);

    Ok(())
}

#[test]
fn self_call_test() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(a, b): (i32, i32), ploxy| {
        ploxy
            .call_univ::<RmpTag, i32>("self", "called", (a, b))
            .unwrap()
    });

    host.host_func::<RmpTag, _, _>("called", |(a, b): (i32, i32), _| a * b);

    let pref = univ.add_plugin("tester", host)?;
    let res = pref.call::<RmpTag, i32>("test", (5, 10))?;

    assert_eq!(res, 50);

    Ok(())
}

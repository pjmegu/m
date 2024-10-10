#![cfg(test)]

use anyhow::*;
use bugi::*;

#[test]
fn host_call() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(name,): (String,)| format!("Hello, {}!", name));

    let pref = univ.add_plugin("host_test", host)?;

    let res = pref.call::<RmpTag, String>("test", ("world".to_string(),))?;

    assert_eq!(res, "Hello, world!");
    Ok(())
}

#[test]
fn cache_test() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(name,): (String,)| format!("Hello, {}!", name));

    let pref = univ.add_plugin("host_test", host)?;

    let cacher = Cacher::new();

    let res = pref.call_cache::<RmpTag, String>("test", ("world".to_string(),), &cacher)?;

    assert_eq!(res, "Hello, world!");

    let res = pref.call_cache::<RmpTag, String>("test", ("world".to_string(),), &cacher)?;

    assert_eq!(res, "Hello, world!");

    Ok(())
}

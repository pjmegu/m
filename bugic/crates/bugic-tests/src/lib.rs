#![cfg(test)]

use anyhow::*;
use bugic::*;

#[test]
fn host_call() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, _, _>("test", |(name,): (String,)| format!("Hello, {}!", name));

    let pref = univ.add_plugin("host_test", host)?;

    let res = pref.call::<RmpTag, (String,), String>("test", ("world".to_string(),))?;

    assert_eq!(res, "Hello, world!");
    Ok(())
}

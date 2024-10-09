use bugic::{host_plug::HostPlugin, Universe};
use bugic_share::RmpTag;

#[test]
fn call_host() {
    let univ = Universe::new();
    let mut host = HostPlugin::new();
    host.host_func::<RmpTag, RmpTag, _, _, _>("hello".to_string(), |param: (String,)| {
        format!("Hello, {}!", param.0)
    });

    let pref = univ
        .add_host_plugin("Hello".to_string(), host)
        .expect("Failed to add plugin");

    let result = pref
        .call::<RmpTag, RmpTag, _, String>("hello".to_string(), ("World".to_string(),))
        .expect("Failed to call plugin");

    assert_eq!(result, "Hello, World!".to_string());
}

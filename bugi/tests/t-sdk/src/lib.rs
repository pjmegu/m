#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bugi_sdk::PluginUniverse;
    use serde::{Deserialize, Serialize};

    #[test]
    fn universe() {
        let _ = PluginUniverse::new();
    }

    #[test]
    fn create_cacher() {
        let univ = PluginUniverse::new();
        let _ = univ.make_cacher();
    }

    const T_1_EMPTY: &[u8] = include_bytes!("../wasms/t_1_empty.wasm");
    #[test]
    fn run_plugin() -> Result<()> {
        let univ = PluginUniverse::new();
        let cacher = univ.make_cacher();
        let pref = univ.load_plugin_from_binary(T_1_EMPTY)?;
        pref.call::<(), ()>(Some(cacher), "func".to_string(), &())?;
        Ok(())
    }

    const T_2_RETURN: &[u8] = include_bytes!("../wasms/t_2_return.wasm");
    #[test]
    fn call_with_res() -> Result<()> {
        let univ = PluginUniverse::new();
        let cacher = univ.make_cacher();
        let pref = univ.load_plugin_from_binary(T_2_RETURN)?;
        let res = pref.call::<(), i32>(Some(cacher), "func".to_string(), &())?;
        assert_eq!(res, 2);
        Ok(())
    }

    const T_3_GET_ARGS: &[u8] = include_bytes!("../wasms/t_3_get_args.wasm");
    #[test]
    fn call_with_args() -> Result<()> {
        let univ = PluginUniverse::new();
        let cacher = univ.make_cacher();
        let pref = univ.load_plugin_from_binary(T_3_GET_ARGS)?;
        let res = pref.call::<(i32, i32), i32>(Some(cacher), "func".to_string(), &(1, 2))?;
        assert_eq!(res, 3);
        Ok(())
    }

    const T_4_CUSTOM_TYPE: &[u8] = include_bytes!("../wasms/t_4_custom_type.wasm");

    #[derive(Serialize)]
    struct Args4 {
        i: i32,
        j: i32,
    }

    #[derive(Serialize)]
    struct Args24 {
        k: i32,
        l: i32,
    }

    #[derive(Deserialize)]
    struct Res4 {
        res: f32,
    }

    #[test]
    fn call_with_custom_type() -> Result<()> {
        let univ = PluginUniverse::new();
        let cacher = univ.make_cacher();
        let pref = univ.load_plugin_from_binary(T_4_CUSTOM_TYPE)?;
        let res = pref.call::<(Args4, Args24), Res4>(
            Some(cacher),
            "func".to_string(),
            &(Args4 { i: 1, j: 2 }, Args24 { k: 3, l: 4 }),
        )?;
        assert_eq!(res.res, ((1 + 3) as f32).sin());
        Ok(())
    }
}

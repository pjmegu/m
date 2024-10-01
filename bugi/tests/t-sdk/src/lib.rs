#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bugi_sdk::PluginUniverse;

    #[test]
    fn universe() {
        let _ = PluginUniverse::new();
    }

    #[test]
    fn create_cacher() {
        let univ = PluginUniverse::new();
        let _ = univ.make_cacher();
    }


    const T_1_EMPTY : &[u8] = include_bytes!("../wasms/t_1_empty.wasm");
    #[test]
    fn run_plugin() -> Result<()> {
        let univ = PluginUniverse::new();
        let cacher = univ.make_cacher();
        let pref = univ.load_plugin_from_binary(T_1_EMPTY)?;
        pref.call::<(), ()>(Some(cacher), "add".to_string(), &())?;
        Ok(())
    }
}
use serde::{de::DeserializeOwned, Serialize};

macro_rules! foreach_func_sig {
    ($mac: ident) => {
        $mac!(P1);
        $mac!(P1, P2);
        $mac!(P1, P2, P3);
        $mac!(P1, P2, P3, P4);
        $mac!(P1, P2, P3, P4, P5);
        $mac!(P1, P2, P3, P4, P5, P6);
        $mac!(P1, P2, P3, P4, P5, P6, P7);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16);
    };
}

/// Type trait that can be set as a plugin argument.
/// The implemented tuple types of [serde::Serialize] can be used.
/// When implementing this, the type must be one that can be serialized into
/// an array at [rmp_serde::to_vec_named] when serializing.
pub trait PluginInput: Serialize {}

impl PluginInput for () {}

macro_rules! gen_plugin_input {
    ($($type:ident),+) => {
        impl<$($type: Serialize),+> PluginInput for ($($type),+,) {}
    };
}

foreach_func_sig!(gen_plugin_input);

/// A trait of the type that will be the return value of the plugin's function.
pub trait PluginResult: DeserializeOwned {}

impl<P: DeserializeOwned> PluginResult for P {}

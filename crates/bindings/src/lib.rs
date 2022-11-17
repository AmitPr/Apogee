mod http_bindings;
pub mod http {
    pub use crate::http_bindings::http_component::*;
    #[cfg(feature = "import")]
    pub mod imports {
        pub use crate::http_bindings::imports::*;
    }
    #[cfg(target_arch = "wasm32")]
    pub use crate::http_bindings::__link_section;
}
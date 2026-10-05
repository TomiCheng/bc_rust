//! cipher 與 key wrapper 共用的參數：[`AnyParams`] 由各自 entry 的 builder 產生。

mod any_params;
mod any_params_builder;

pub use any_params::AnyParams;
pub use any_params_builder::AnyParamsBuilder;
pub(crate) use any_params_builder::ParamsRule;

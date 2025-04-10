pub mod date_utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionType {
    Call, Put
}
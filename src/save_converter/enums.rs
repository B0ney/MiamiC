#[derive(Debug, std::cmp::PartialEq, Clone)]
pub enum SaveType {
    Unknown,
    Retail,
    Steam,
    Android, // unsupported right now
    IOS,     // unsupported right now
}

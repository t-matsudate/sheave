#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublisherStatus {
    #[default]
    Connected,
    Released,
    FcPublished,
    Created,
    Began,
    Published,
    Other
}

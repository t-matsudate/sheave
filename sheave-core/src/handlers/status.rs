/// The pattern of communication status between servers and publisher clients.
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

/// The pattern of communication status between servers and subscriber clients.
///
/// Note there are several subscribing tools which will sent different commands between FCSubscribe and StreamBegin.
/// For example:
///
/// * FFmpeg: getStreamLength
/// * OBS: set_playlist
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubscriberStatus {
    #[default]
    Connected,
    WindowAcknowledgementSizeGotSent,
    Created,
    FcSubscribed,
    AdditionalCommandGotSent,
    Began,
    Played,
    BufferLengthGotSent,
    Other
}

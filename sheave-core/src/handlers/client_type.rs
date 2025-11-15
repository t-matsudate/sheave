/// The pattern to distinguish whether client requests are publishers or subscribers.
///
/// We are required to note that RTMP is one of Publish-Subscribe model.
/// Notably, RTMP servers are requried to distinguish whether they are publishers or subscribers by states of data sent.
///
/// Currently, this is used by servers when distinguished them successfully.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClientType {
    #[default]
    Publisher,
    Subscriber
}

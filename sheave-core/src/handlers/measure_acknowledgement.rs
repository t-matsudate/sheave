use crate::messages::Acknowledgement;

/// Measurement receiving chunk size.
///
/// Clients and the server are each required to send [`Acknowledgement`] messages when received chunk sizes exceed their bandwidths.
/// This trait defines ways to measure receiving chunk sizes for any stream.
pub trait MeasureAcknowledgement {
    /// Resets measured count and turns the flag into on, for beginning measurement.
    fn begin_measuring(&mut self);

    /// Resets measured count and turns the flag into off, for finishing measurement.
    fn finish_measuring(&mut self);

    /// Adds received chunk size to current one.
    fn add_amount(&mut self, amount: u32);

    /// Gets current chunk size.
    fn get_current_amount(&mut self) -> u32;

    /// Wraps current chunk size into an [`Acknowledgement`] message.
    ///
    /// [`Acknowledgement`]: crate::messages::Acknowledgement
    fn as_acknowledgement(&mut self) -> Acknowledgement {
        Acknowledgement::new(self.get_current_amount())
    }
}

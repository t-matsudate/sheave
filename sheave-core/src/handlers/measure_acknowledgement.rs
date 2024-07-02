use crate::messages::Acknowledgement;

pub trait MeasureAcknowledgement {
    fn begin_measuring(&mut self);
    fn finish_measuring(&mut self);
    fn add_amount(&mut self, amount: u32);
    fn get_current_amount(&mut self) -> u32;

    fn as_acknowledgement(&mut self) -> Acknowledgement {
        Acknowledgement::new(self.get_current_amount())
    }
}

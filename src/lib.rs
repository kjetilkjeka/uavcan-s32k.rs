#![no_std]

extern crate uavcan;
extern crate s32k144;
#[macro_use]
extern crate s32k144evb;
extern crate embedded_types;

use core::cell::RefCell;

use uavcan::transfer::TransferFrame;
use uavcan::transfer::TransferFrameID;
use uavcan::transfer::TransferID;
use uavcan::transfer::TransferInterface;
use uavcan::transfer::FullTransferID;
use uavcan::transfer::IOError;

use embedded_types::can::ExtendedDataFrame as CanFrame;

pub struct Interface<'a> {
    interface: s32k144evb::can::Can<'a>,
    rx_buffer: RefCell<ReceiveBuffer>,
}

impl<'a> Interface<'a> {
    pub fn new(
        can: &'a s32k144::can0::RegisterBlock,
        spc: &'a s32k144evb::spc::Spc<'a>,
    ) -> Self {
        let mut can_settings = s32k144evb::can::CanSettings::default();    
        can_settings.self_reception = false;

        let can_interface = s32k144evb::can::Can::init(can, spc, &can_settings).unwrap();
        
        Interface{
            interface: can_interface,
            rx_buffer: RefCell::new(ReceiveBuffer::new()),
        }
    }
}

impl<'a> TransferInterface<'a> for Interface<'a> {
    type Frame = CanFrame;

    fn transmit(&self, frame: &CanFrame) -> Result<(), IOError> {
        self.interface.transmit_quick(&embedded_types::can::CanFrame::from(*frame))
    }
    
    fn receive(&self, identifier: &FullTransferID) -> Option<CanFrame> {
        while let Ok(frame) = self.interface.receive() {
            if let embedded_types::can::CanFrame::DataFrame(embedded_types::can::DataFrame::ExtendedDataFrame(edf)) = frame {
                self.rx_buffer.borrow_mut().push(edf);
            }
        }

        self.rx_buffer.borrow_mut().remove(identifier)
    }

    fn completed_receive(&self, identifier: FullTransferID, mask: FullTransferID) -> Option<FullTransferID> {
        while let Ok(frame) = self.interface.receive() {
            if let embedded_types::can::CanFrame::DataFrame(embedded_types::can::DataFrame::ExtendedDataFrame(edf)) = frame {
                self.rx_buffer.borrow_mut().push(edf);
            }
        }
        
        self.rx_buffer.borrow_mut().completed_receive(identifier, mask)
    }    
}

struct ReceiveBuffer{
    buffer: [CanFrame; ReceiveBuffer::BUFFER_LENGTH],
    length: usize,
    overflow: bool,
}

impl ReceiveBuffer{
    const BUFFER_LENGTH: usize = 20;
    
    pub fn new() -> Self {
        ReceiveBuffer{
            buffer: [CanFrame::new(embedded_types::can::ExtendedID::new(0)); Self::BUFFER_LENGTH],
            length: 0,
            overflow: false,
        }
    }

    pub fn push(&mut self, frame: CanFrame) {
        if self.length == ReceiveBuffer::BUFFER_LENGTH {
            self.remove_index(0);
            self.overflow = true;
        }
        self.buffer[self.length] = frame;
        self.length += 1;
    }

    pub fn remove(&mut self, identifier: &FullTransferID) -> Option<CanFrame> {
        let opt_index = self.buffer[0..self.length].iter().position(|value| value.full_id() == *identifier);

        match opt_index {
            Some(index) => Some(self.remove_index(index)),
            None => None,
        }
    }

    fn remove_index(&mut self, index: usize) -> CanFrame {
        let frame = self.buffer[index];
        self.length -= 1;
        for i in index..self.length {
            self.buffer[i] = self.buffer[i+1];
        }
        frame
    }

    pub fn completed_receive(&mut self, identifier: FullTransferID, mask: FullTransferID) -> Option<FullTransferID> {
        self.buffer[0..self.length].iter()
            .filter(|frame| frame.data().len() >= 1)
            .filter(|frame| frame.full_id().mask(mask) == identifier.mask(mask))
            .filter(|frame| frame.is_end_frame())
            .map(|frame| frame.full_id())
            .next()            
    }
    
}


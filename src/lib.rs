#![feature(alloc)]
#![no_std]

extern crate uavcan;
extern crate s32k144;
extern crate s32k144evb;
extern crate embedded_types;
extern crate alloc;

use uavcan::transfer::TransferFrame;
use uavcan::transfer::TransferInterface;
use uavcan::transfer::FullTransferID;
use uavcan::transfer::IOError;

use alloc::Vec;

use embedded_types::can::ExtendedDataFrame as CanFrame;

pub struct Interface<'a> {
    interface: &'a s32k144::can0::RegisterBlock,
}

impl<'a> Interface<'a> {
    pub fn new(can: &'a s32k144::can0::RegisterBlock) -> Self {
        Interface{interface: can}
    }
}

impl<'a> TransferInterface<'a> for Interface<'a> {
    type Frame = CanFrame;
    type IDContainer = &'a [FullTransferID];

    fn transmit(&self, frame: &CanFrame) -> Result<(), IOError> {unimplemented!()}
    fn receive(&self, identifier: &FullTransferID) -> Option<CanFrame> {unimplemented!()}
    fn completed_receives(&self, identifier: FullTransferID, mask: FullTransferID) -> Self::IDContainer {unimplemented!()}    
}


#![no_std]
#![feature(alloc)]
extern crate alloc;
extern crate uavcan;
extern crate s32k144;
#[macro_use]
extern crate s32k144evb;
extern crate embedded_types;
extern crate cortex_m;

use core::cell::RefCell;

use alloc::vec::Vec;
use alloc::arc::Arc;

use cortex_m::interrupt::Mutex;

use uavcan::transfer::TransferFrame;
use uavcan::transfer::TransferSubscriber;
use uavcan::transfer::TransferFrameID;
use uavcan::transfer::TransferFrameIDFilter;
use uavcan::transfer::TransferID;
use uavcan::transfer::TransferInterface;
use uavcan::transfer::FullTransferID;
use uavcan::transfer::IOError;

use embedded_types::can::ExtendedDataFrame as CanFrame;

pub struct Interface<'a> {
    interface: s32k144evb::can::Can<'a>,
    subscribers: Mutex<RefCell<Vec<SubscriberHandle>>>,
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
            subscribers: Mutex::new(RefCell::new(Vec::new())),
        }
    }

    pub fn spin(&self) {
        while let Ok(frame) = self.interface.receive() {
            if let embedded_types::can::CanFrame::DataFrame(embedded_types::can::DataFrame::ExtendedDataFrame(edf)) = frame {
                cortex_m::interrupt::free(|cs| {
                    for sub in self.subscribers.borrow(&cs).borrow_mut().iter() {
                        if sub.filter.is_match(TransferFrame::id(&edf)) {
                            let mut buffer = sub.buffer.borrow(&cs).borrow_mut();
                            buffer.push(edf);
                        }
                    }
                });
            }
        }
    }
}

impl<'a> TransferInterface for Interface<'a> {
    type Frame = CanFrame;
    type Subscriber = Subscriber;

    fn transmit(&self, frame: &CanFrame) -> Result<(), IOError> {
        self.interface.transmit_quick(&embedded_types::can::CanFrame::from(*frame))
    }

    fn subscribe(&self, filter: TransferFrameIDFilter) -> Result<Self::Subscriber, ()> {
        let buffer = Arc::new(Mutex::new(RefCell::new(Vec::new())));
        cortex_m::interrupt::free(|cs| {
            let mut subscribers = self.subscribers.borrow(&cs).borrow_mut();
        
            let subscriber_handle = SubscriberHandle {
                filter: filter,
                buffer: buffer.clone(),
            };
        
            subscribers.push(subscriber_handle);
        
            Ok(Subscriber{
                buffer: buffer,
            })
        })
    }
    
}


pub struct SubscriberHandle {
    buffer: Arc<Mutex<RefCell<Vec<CanFrame>>>>,
    filter: TransferFrameIDFilter,
}

pub struct Subscriber {
    buffer: Arc<Mutex<RefCell<Vec<CanFrame>>>>,
}

impl TransferSubscriber for Subscriber {
    type Frame = CanFrame;

    fn receive(&self, identifier: &TransferFrameID) -> Option<Self::Frame> {
        cortex_m::interrupt::free(|cs| {
            let mut buffer = self.buffer.borrow(&cs).borrow_mut();
            let pos = buffer.iter().position(|x| TransferFrame::id(x) == *identifier)?;
            Some(buffer.remove(pos))
        })
    }

    fn retain<F>(&self, f: F) where F: FnMut(&Self::Frame) -> bool {
        cortex_m::interrupt::free(|cs| {
            let mut buffer = self.buffer.borrow(cs).borrow_mut();
            buffer.retain(f);
        });
    }
    
    fn find<F>(&self, mut predicate: F) -> Option<Self::Frame> where F: FnMut(&Self::Frame) -> bool {
        cortex_m::interrupt::free(|cs| {
            let buffer = self.buffer.borrow(&cs).borrow();
            Some(*buffer.iter().find(|x| predicate(&x))?)
        })
    }
    
}

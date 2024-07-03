use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

use crate::packet;


#[context_macro]
pub struct kernel<A: Clone> {
    pub in_stream: Receiver<packet>,
    pub out_stream: Sender<packet>,
    pub latency: usize,
    pub init_inverval: usize,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> kernel<A>
where
kernel<A>: Context,
{
    pub fn new(
        in_stream: Receiver<packet>,
        out_stream: Sender<packet>,
        latency: usize,
        init_inverval: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let kernel = kernel {
            in_stream,
            out_stream,
            latency,
            init_inverval,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        kernel.in_stream.attach_receiver(&kernel);
        kernel.out_stream.attach_sender(&kernel);

        kernel
    }
}

impl<A: DAMType + num::Num> Context for kernel<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in1 = self.in_stream.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time + self.latency as u64, in1.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(self.init_inverval as u64);
        }
    }
}

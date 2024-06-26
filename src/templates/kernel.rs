use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};


#[context_macro]
pub struct kernel<A: Clone> {
    pub in_stream: Receiver<A>,
    pub out_stream: Sender<A>,
    pub latency: usize,
    pub init_inverval: usize,
    pub loop_bound: usize,
}

impl<A: DAMType> kernel<A>
where
kernel<A>: Context,
{
    pub fn new(
        in_stream: Receiver<A>,
        out_stream: Sender<A>,
        latency: usize,
        init_inverval: usize,
        loop_bound: usize,
    ) -> Self {
        let ker = kernel {
            in_stream,
            out_stream,
            latency,
            init_inverval,
            loop_bound,
            context_info: Default::default(),
        };
        ker.in_stream.attach_receiver(&ker);
        ker.out_stream.attach_sender(&ker);

        ker
    }
}

impl<A: DAMType + num::Num> Context for kernel<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in1: Result<ChannelElement<A>, dam::channel::DequeueError> = self.in_stream.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time + self.latency as u64, in1.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(self.init_inverval as u64);
        }
    }
}

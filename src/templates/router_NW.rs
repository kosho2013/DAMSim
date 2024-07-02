use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_NW<A: Clone> {
    pub in_N: Receiver<usize>,
    pub in_W: Receiver<usize>,
    pub out_N: Sender<usize>,
    pub out_W: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_NW<A>
where
router_NW<A>: Context,
{
    pub fn new(
        in_N: Receiver<usize>,
        in_W: Receiver<usize>,
        out_N: Sender<usize>,
        out_W: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_NW = router_NW {
            in_N,
            in_W,
            out_N,
            out_W,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_NW.in_N.attach_receiver(&router_NW);
        router_NW.in_W.attach_receiver(&router_NW);
        router_NW.out_N.attach_sender(&router_NW);
        router_NW.out_W.attach_sender(&router_NW);

        router_NW
    }
}

impl<A: DAMType + num::Num> Context for router_NW<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_N= self.in_N.dequeue(&self.time);
            let in_W= self.in_W.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_N.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_N.unwrap().data.clone())).unwrap();
            self.out_W.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_W.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}
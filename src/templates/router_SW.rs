use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_SW<A: Clone> {
    pub in_S: Receiver<usize>,
    pub in_W: Receiver<usize>,
    pub out_S: Sender<usize>,
    pub out_W: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_SW<A>
where
router_SW<A>: Context,
{
    pub fn new(
        in_S: Receiver<usize>,
        in_W: Receiver<usize>,
        out_S: Sender<usize>,
        out_W: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_SW = router_SW {
            in_S,
            in_W,
            out_S,
            out_W,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_SW.in_S.attach_receiver(&router_SW);
        router_SW.in_W.attach_receiver(&router_SW);
        router_SW.out_S.attach_sender(&router_SW);
        router_SW.out_W.attach_sender(&router_SW);

        router_SW
    }
}

impl<A: DAMType + num::Num> Context for router_SW<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_S= self.in_S.dequeue(&self.time);
            let in_W= self.in_W.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_S.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_S.unwrap().data.clone())).unwrap();
            self.out_W.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_W.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}
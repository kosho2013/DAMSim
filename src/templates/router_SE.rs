use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_SE<A: Clone> {
    pub in_S: Receiver<usize>,
    pub in_E: Receiver<usize>,
    pub out_S: Sender<usize>,
    pub out_E: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_SE<A>
where
router_SE<usize>: Context,
{
    pub fn new(
        in_S: Receiver<usize>,
        in_E: Receiver<usize>,
        out_S: Sender<usize>,
        out_E: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_SE = router_SE {
            in_S,
            in_E,
            out_S,
            out_E,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_SE.in_S.attach_receiver(&router_SE);
        router_SE.in_E.attach_receiver(&router_SE);
        router_SE.out_S.attach_sender(&router_SE);
        router_SE.out_E.attach_sender(&router_SE);

        router_SE
    }
}

impl<A: DAMType + num::Num> Context for router_SE<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_S= self.in_S.dequeue(&self.time);
            let in_E= self.in_E.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_S.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_S.unwrap().data.clone())).unwrap();
            self.out_E.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_E.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}
use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_NEW<A: Clone> {
    pub in_N: Receiver<usize>,
    pub in_E: Receiver<usize>,
    pub in_W: Receiver<usize>,
    pub out_N: Sender<usize>,
    pub out_E: Sender<usize>,
    pub out_W: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_NEW<A>
where
router_NEW<usize>: Context,
{
    pub fn new(
        in_N: Receiver<usize>,
        in_E: Receiver<usize>,
        in_W: Receiver<usize>,
        out_N: Sender<usize>,
        out_E: Sender<usize>,
        out_W: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_NEW = router_NEW {
            in_N,
            in_E,
            in_W,
            out_N,
            out_E,
            out_W,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_NEW.in_N.attach_receiver(&router_NEW);
        router_NEW.in_E.attach_receiver(&router_NEW);
        router_NEW.in_W.attach_receiver(&router_NEW);
        router_NEW.out_N.attach_sender(&router_NEW);
        router_NEW.out_E.attach_sender(&router_NEW);
        router_NEW.out_W.attach_sender(&router_NEW);

        router_NEW
    }
}

impl<A: DAMType + num::Num> Context for router_NEW<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_N = self.in_N.dequeue(&self.time);
            let in_E = self.in_E.dequeue(&self.time);
            let in_W = self.in_W.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_N.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_N.unwrap().data.clone())).unwrap();
            self.out_E.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_E.unwrap().data.clone())).unwrap();
            self.out_W.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_W.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}
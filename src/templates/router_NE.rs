use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_NE<A: Clone> {
    pub in_N: Receiver<usize>,
    pub in_E: Receiver<usize>,
    pub out_N: Sender<usize>,
    pub out_E: Sender<usize>,
    pub in_local: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_local: Vec<Sender<usize>>,
    pub out_len: usize,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_NE<A>
where
router_NE<A>: Context,
{
    pub fn new(
        in_N: Receiver<usize>,
        in_E: Receiver<usize>,
        out_N: Sender<usize>,
        out_E: Sender<usize>,
        in_local: Vec<Receiver<usize>>,
        in_len: usize,
        out_local: Vec<Sender<usize>>,
        out_len: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_NE = router_NE {
            in_N,
            in_E,
            out_N,
            out_E,
            in_local,
            in_len,
            out_local,
            out_len,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_NE.in_N.attach_receiver(&router_NE);
        router_NE.in_E.attach_receiver(&router_NE);
        router_NE.out_N.attach_sender(&router_NE);
        router_NE.out_E.attach_sender(&router_NE);

        for i in 0..in_len
        {
            router_NE.in_local[i].attach_receiver(&router_NE);
        }
        
        for i in 0..out_len
        {
            router_NE.out_local[i].attach_sender(&router_NE);
        }

        router_NE
    }
}

impl<A: DAMType + num::Num> Context for router_NE<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_N = self.in_N.dequeue(&self.time);
            let in_E = self.in_E.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_N.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_N.unwrap().data.clone())).unwrap();
            self.out_E.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_E.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}
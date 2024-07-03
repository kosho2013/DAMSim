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
    pub in_local: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_local: Vec<Sender<usize>>,
    pub out_len: usize,
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
        in_local: Vec<Receiver<usize>>,
        in_len: usize,
        out_local: Vec<Sender<usize>>,
        out_len: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_SW = router_SW {
            in_S,
            in_W,
            out_S,
            out_W,
            in_local,
            in_len,
            out_local,
            out_len,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_SW.in_S.attach_receiver(&router_SW);
        router_SW.in_W.attach_receiver(&router_SW);
        router_SW.out_S.attach_sender(&router_SW);
        router_SW.out_W.attach_sender(&router_SW);

        for i in 0..in_len
        {
            router_SW.in_local[i].attach_receiver(&router_SW);
        }
        
        for i in 0..out_len
        {
            router_SW.out_local[i].attach_sender(&router_SW); 
        }

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
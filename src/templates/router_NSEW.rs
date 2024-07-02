use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router_NSEW<A: Clone> {
    pub in_N: Receiver<usize>,
    pub in_S: Receiver<usize>,
    pub in_E: Receiver<usize>,
    pub in_W: Receiver<usize>,
    pub out_N: Sender<usize>,
    pub out_S: Sender<usize>,
    pub out_E: Sender<usize>,
    pub out_W: Sender<usize>,
    pub in_self: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_self: Vec<Sender<usize>>,
    pub out_len: usize,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router_NSEW<A>
where
router_NSEW<A>: Context,
{
    pub fn new(
        in_N: Receiver<usize>,
        in_S: Receiver<usize>,
        in_E: Receiver<usize>,
        in_W: Receiver<usize>,
        out_N: Sender<usize>,
        out_S: Sender<usize>,
        out_E: Sender<usize>,
        out_W: Sender<usize>,
        in_self: Vec<Receiver<usize>>,
        in_len: usize,
        out_self: Vec<Sender<usize>>,
        out_len: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_NSEW = router_NSEW {
            in_N,
            in_S,
            in_E,
            in_W,
            out_N,
            out_S,
            out_E,
            out_W,
            in_self,
            in_len,
            out_self,
            out_len,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        router_NSEW.in_N.attach_receiver(&router_NSEW);
        router_NSEW.in_S.attach_receiver(&router_NSEW);
        router_NSEW.in_E.attach_receiver(&router_NSEW);
        router_NSEW.in_W.attach_receiver(&router_NSEW);
        router_NSEW.out_N.attach_sender(&router_NSEW);
        router_NSEW.out_S.attach_sender(&router_NSEW);
        router_NSEW.out_E.attach_sender(&router_NSEW);
        router_NSEW.out_W.attach_sender(&router_NSEW);

        for i in 0..in_len
        {
            router_NSEW.in_self[i].attach_receiver(&router_NSEW);
        }
        
        for i in 0..out_len
        {
            router_NSEW.out_self[i].attach_sender(&router_NSEW);
        }

        router_NSEW
    }
}

impl<A: DAMType + num::Num> Context for router_NSEW<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_N= self.in_N.dequeue(&self.time);
            let in_S= self.in_S.dequeue(&self.time);
            let in_E= self.in_E.dequeue(&self.time);
            let in_W= self.in_W.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_N.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_S.unwrap().data.clone())).unwrap();
            self.out_S.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_N.unwrap().data.clone())).unwrap();
            self.out_E.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_W.unwrap().data.clone())).unwrap();
            self.out_W.enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_E.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}
use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct to_router_adapter<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType> to_router_adapter<A>
where
to_router_adapter<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let to_router_adapter = to_router_adapter {
            in_stream,
            in_len,
            out_stream,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            to_router_adapter.in_stream[idx].attach_receiver(&to_router_adapter);
        }
        to_router_adapter.out_stream.attach_sender(&to_router_adapter);

        to_router_adapter
    }
}

impl<A: DAMType + num::Num> Context for to_router_adapter<A> {
    fn run(&mut self) {
        for i in 0..self.loop_bound
        {
            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                let in_data = self.in_stream[idx].dequeue(&self.time).unwrap().data;
                
                println!("xxxxxxxx");

                let curr_time = self.time.tick();
                self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
                self.time.incr_cycles(1);
            }
        }
    }
}








#[context_macro]
pub struct from_router_adapter<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub loop_bound: usize,
    pub dummy: A,
}

impl<A: DAMType> from_router_adapter<A>
where
from_router_adapter<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let from_router_adapter = from_router_adapter {
            in_stream,
            out_stream,
            out_len,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        from_router_adapter.in_stream.attach_receiver(&from_router_adapter);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            from_router_adapter.out_stream[idx].attach_sender(&from_router_adapter);
        }

        from_router_adapter
    }
}

impl<A: DAMType + num::Num> Context for from_router_adapter<A> {
    fn run(&mut self) {
        for i in 0..self.loop_bound
        {
            for j in 0..self.out_len
            {
                let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

                let curr_time = self.time.tick();
                let idx: usize = j.try_into().unwrap();
                self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
                self.time.incr_cycles(1);
            }
        }
    }
}
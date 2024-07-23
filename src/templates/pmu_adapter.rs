use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct pmu_adapter_upstream<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream_wr_addr: Sender<usize>,
    pub out_stream_wr_data: Sender<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> pmu_adapter_upstream<A>
where
pmu_adapter_upstream<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream_wr_addr: Sender<usize>,
        out_stream_wr_data: Sender<usize>,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let pmu_adapter_upstream = pmu_adapter_upstream {
            in_stream,
            in_len,
            out_stream_wr_addr,
            out_stream_wr_data,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            pmu_adapter_upstream.in_stream[idx].attach_receiver(&pmu_adapter_upstream);
        }
        pmu_adapter_upstream.out_stream_wr_addr.attach_sender(&pmu_adapter_upstream);
        pmu_adapter_upstream.out_stream_wr_data.attach_sender(&pmu_adapter_upstream);

        pmu_adapter_upstream
    }
}

impl<A: DAMType + num::Num> Context for pmu_adapter_upstream<A> {
    fn run(&mut self) {
        let mut cnt: usize = 0;
        for _ in 0..self.num_input {
            let mut in_vec = vec![];

            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                in_vec.push(self.in_stream[idx].dequeue(&self.time));
            }

            let in_data = in_vec.remove(0).unwrap().data;

            for _ in 0..self.counter
            {
                let curr_time = self.time.tick();
                self.out_stream_wr_addr.enqueue(&self.time, ChannelElement::new(curr_time + 1, 0)).unwrap();
                self.out_stream_wr_data.enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
                cnt += 1;
            }
        }
        self.time.incr_cycles(1);
    }
}








#[context_macro]
pub struct pmu_adapter_downstream<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub num_input: usize,
    pub counter: usize,
    pub dummy: A,
}

impl<A: DAMType> pmu_adapter_downstream<A>
where
pmu_adapter_downstream<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        out_dst: Vec<usize>,
        num_input: usize,
        counter: usize,
        dummy: A,
    ) -> Self {
        let pmu_adapter_downstream = pmu_adapter_downstream {
            in_stream,
            out_stream,
            out_len,
            out_dst,
            num_input,
            counter,
            dummy,
            context_info: Default::default(),
        };
        pmu_adapter_downstream.in_stream.attach_receiver(&pmu_adapter_downstream);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            pmu_adapter_downstream.out_stream[idx].attach_sender(&pmu_adapter_downstream);
        }

        pmu_adapter_downstream
    }
}

impl<A: DAMType + num::Num> Context for pmu_adapter_downstream<A> {
    fn run(&mut self) {
        let tmp = self.counter * self.num_input; 
        for i in 0..tmp
        {
            let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

            if i % self.counter == 0
            {
                for j in 0..self.out_len
                {
                    let curr_time = self.time.tick();
                    let idx: usize = j.try_into().unwrap();
                    self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time + 1, self.out_dst[j])).unwrap();
                }
            }
        }
        self.time.incr_cycles(1);
    }
}
use std::mem;

use dam::context_tools::*;
use dam::types::StaticallySized;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct simd_pcu_adapter_upstream<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream: Sender<usize>,
    pub num_input: usize,
    pub m: usize,
    pub k: usize,
    pub n: usize,
    pub dummy: A,
}

impl<A: DAMType> simd_pcu_adapter_upstream<A>
where
simd_pcu_adapter_upstream<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream: Sender<usize>,
        num_input: usize,
        m: usize,
        k: usize,
        n: usize,
        dummy: A,
    ) -> Self {
        let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream {
            in_stream,
            in_len,
            out_stream,
            num_input,
            m,
            k,
            n,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            simd_pcu_adapter_upstream.in_stream[idx].attach_receiver(&simd_pcu_adapter_upstream);
        }
        simd_pcu_adapter_upstream.out_stream.attach_sender(&simd_pcu_adapter_upstream);

        simd_pcu_adapter_upstream
    }
}

impl<A: DAMType + num::Num> Context for simd_pcu_adapter_upstream<A> {
    fn run(&mut self) {
        let tmp = self.m * self.k * self.n;
        for _ in 0..self.num_input {
            let mut in_vec = vec![];

            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                in_vec.push(self.in_stream[idx].dequeue(&self.time));
            }

            let in_data = in_vec.remove(0).unwrap().data;

            for _ in 0..tmp
            {
                let curr_time = self.time.tick();
                self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
            }
        }
        self.time.incr_cycles(1);
    }
}






















#[context_macro]
pub struct simd_pcu_adapter_downstream<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub num_input: usize,
    pub m: usize,
    pub k: usize,
    pub n: usize,
    pub dummy: A,
}

impl<A: DAMType> simd_pcu_adapter_downstream<A>
where
simd_pcu_adapter_downstream<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        out_dst: Vec<usize>,
        num_input: usize,
        m: usize,
        k: usize,
        n: usize,
        dummy: A,
    ) -> Self {
        let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream {
            in_stream,
            out_stream,
            out_len,
            out_dst,
            num_input,
            m,
            k,
            n,
            dummy,
            context_info: Default::default(),
        };
        simd_pcu_adapter_downstream.in_stream.attach_receiver(&simd_pcu_adapter_downstream);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            simd_pcu_adapter_downstream.out_stream[idx].attach_sender(&simd_pcu_adapter_downstream);
        }

        simd_pcu_adapter_downstream
    }
}

impl<A: DAMType + num::Num> Context for simd_pcu_adapter_downstream<A> {
    fn run(&mut self) {
        let tmp = self.m * self.k * self.n * self.num_input;
        let tmp2 = self.m * self.k * self.n;
        for i in 0..tmp
        {
            let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

            if i % tmp2 == 0
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














#[context_macro]
pub struct systolic_pcu_adapter_upstream<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream_lane: Sender<usize>,
    pub out_stream_stage: Sender<usize>,
    pub num_input: usize,
    pub m: usize,
    pub k: usize,
    pub n: usize,
    pub lane_dim: usize,
    pub stage_dim: usize,
    pub dummy: A,
}

impl<A: DAMType> systolic_pcu_adapter_upstream<A>
where
systolic_pcu_adapter_upstream<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream_lane: Sender<usize>,
        out_stream_stage: Sender<usize>,
        num_input: usize,
        m: usize,
        k: usize,
        n: usize,
        lane_dim: usize,
        stage_dim: usize,
        dummy: A,
    ) -> Self {
        let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream {
            in_stream,
            in_len,
            out_stream_lane,
            out_stream_stage,
            num_input,
            m,
            k,
            n,
            lane_dim,
            stage_dim,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            systolic_pcu_adapter_upstream.in_stream[idx].attach_receiver(&systolic_pcu_adapter_upstream);
        }
        systolic_pcu_adapter_upstream.out_stream_lane.attach_sender(&systolic_pcu_adapter_upstream);
        systolic_pcu_adapter_upstream.out_stream_stage.attach_sender(&systolic_pcu_adapter_upstream);

        systolic_pcu_adapter_upstream
    }
}

impl<A: DAMType + num::Num> Context for systolic_pcu_adapter_upstream<A> {
    fn run(&mut self) {
        let tmp_inner = self.m * (self.k) * self.n;
        for _ in 0..self.num_input {
            let mut in_vec = vec![];

            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                in_vec.push(self.in_stream[idx].dequeue(&self.time));
            }

            let in_data = in_vec.remove(0).unwrap().data;

            for i in 0..tmp_inner
            {
                let curr_time = self.time.tick();
                self.out_stream_lane.enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
                self.out_stream_stage.enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
            }
        }
        self.time.incr_cycles(1);
    }
}








#[context_macro]
pub struct systolic_pcu_adapter_downstream<A: Clone> {
    pub in_stream_lane: Receiver<usize>,
    pub in_stream_stage: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub num_input: usize,
    pub m: usize,
    pub k: usize,
    pub n: usize,
    pub lane_dim: usize,
    pub stage_dim: usize,
    pub dummy: A,
}

impl<A: DAMType> systolic_pcu_adapter_downstream<A>
where
systolic_pcu_adapter_downstream<A>: Context,
{
    pub fn new(
        in_stream_lane: Receiver<usize>,
        in_stream_stage: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        out_dst: Vec<usize>,
        num_input: usize,
        m: usize,
        k: usize,
        n: usize,
        lane_dim: usize,
        stage_dim: usize,
        dummy: A,
    ) -> Self {
        let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream {
            in_stream_lane,
            in_stream_stage,
            out_stream,
            out_len,
            out_dst,
            num_input,
            m,
            k,
            n,
            lane_dim,
            stage_dim,
            dummy,
            context_info: Default::default(),
        };
        systolic_pcu_adapter_downstream.in_stream_lane.attach_receiver(&systolic_pcu_adapter_downstream);
        systolic_pcu_adapter_downstream.in_stream_stage.attach_receiver(&systolic_pcu_adapter_downstream);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            systolic_pcu_adapter_downstream.out_stream[idx].attach_sender(&systolic_pcu_adapter_downstream);
        }

        systolic_pcu_adapter_downstream
    }
}

impl<A: DAMType + num::Num> Context for systolic_pcu_adapter_downstream<A> {
    fn run(&mut self) {
        let tmp_outer = self.m * self.k * self.n * self.num_input;
        let tmp_inner = self.m * self.k * self.n;
        for i in 0..tmp_outer
        {
            let in_lane = self.in_stream_lane.dequeue(&self.time);
            let in_stage = self.in_stream_stage.dequeue(&self.time);
            if (i % tmp_inner == 0)
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
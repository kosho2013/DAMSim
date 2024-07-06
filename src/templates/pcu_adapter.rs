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
    pub loop_bound: usize,
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
        loop_bound: usize,
        m: usize,
        k: usize,
        n: usize,
        dummy: A,
    ) -> Self {
        let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream {
            in_stream,
            in_len,
            out_stream,
            loop_bound,
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

        for _ in 0..self.loop_bound {
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
                self.time.incr_cycles(1);
            }
        }
    }
}






















#[context_macro]
pub struct simd_pcu_adapter_downstream<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub loop_bound: usize,
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
        loop_bound: usize,
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
            loop_bound,
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
        let tmp = self.m * self.k * self.n * self.loop_bound;
        let tmp2 = self.m * self.k * self.n;
        for i in 0..tmp
        {
            let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

            if (i % tmp2 == 0)
            {
                for j in 0..self.out_len
                {
                    let curr_time = self.time.tick();
                    let idx: usize = j.try_into().unwrap();
                    self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time + 1, self.out_dst[j])).unwrap();
                    self.time.incr_cycles(1);

                    println!("zzzzzzzzzzzzzzzzzzzzzzzzz");
                }   
            }
        }
    }
}














#[context_macro]
pub struct systolic_pcu_adapter_upstream<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream_lane: Sender<usize>,
    pub out_stream_stage: Sender<usize>,
    pub loop_bound: usize,
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
        loop_bound: usize,
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
            loop_bound,
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

        for _ in 0..self.loop_bound {
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
                self.time.incr_cycles(1);
            }
        }
    }
}








#[context_macro]
pub struct systolic_pcu_adapter_downstream<A: Clone> {
    pub in_stream_lane: Receiver<usize>,
    pub in_stream_stage: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_dst: Vec<usize>,
    pub loop_bound: usize,
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
        loop_bound: usize,
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
            loop_bound,
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
        let tmp_outer = self.m * (self.k) * self.n * self.loop_bound;
        let tmp_inner = self.m * (self.k) * self.n;

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
                    self.time.incr_cycles(1);
                }   
            }
        }

    }
}








#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dam::{
        shim::RunMode, simulation::{DotConvertible, InitializationOptions, InitializationOptionsBuilder, ProgramBuilder, RunOptions, RunOptionsBuilder}, templates::{ops::{ALUAddOp, ALUMulOp}, pcu::{PCUConfig, PipelineStage, PCU}}, utility_contexts::{CheckerContext, GeneratorContext, PrinterContext}
    };

    use crate::templates::{my_pcu::{make_simd_pcu, make_systolic_pcu}, pcu_adapter::{simd_pcu_adapter_downstream, simd_pcu_adapter_upstream, systolic_pcu_adapter_downstream, systolic_pcu_adapter_upstream}};
    
    // #[test]
    // fn test_simd_pcu_adapter_upstream()
    // {
    //     let mut parent = ProgramBuilder::default();
    //     let mut my_vec = vec![];

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator2 = GeneratorContext::new(iter, sender2);
    //     parent.add_child(generator2);


    //     my_vec.push(receiver1);
    //     my_vec.push(receiver2);

        
    //     // printer contexts
    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let printer = PrinterContext::new(receiver2);
    //     parent.add_child(printer);




    //     let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(my_vec, 2, sender2, 100, 2, 2, 2);
    //     parent.add_child(simd_pcu_adapter_upstream);












    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }








    // #[test]
    // fn test_simd_pcu_adapter_downstream()
    // {
    //     let mut parent = ProgramBuilder::default();
    //     let mut my_vec = vec![];

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let iter = || (0..800).map(|i| (i as i32) * 1_i32);
    //     let generator = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator);
        
    //     // printer contexts
    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let printer = PrinterContext::new(receiver2);
    //     parent.add_child(printer);

    //     let (sender3, receiver3) = parent.bounded(1024);
    //     let printer = PrinterContext::new(receiver3);
    //     parent.add_child(printer);

    //     my_vec.push(sender2);
    //     my_vec.push(sender3);

    //     let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver1, my_vec, 2, 100, 2, 2, 2);
    //     parent.add_child(simd_pcu_adapter_downstream);












    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }








    // #[test]
    // fn test_simd_pcu()
    // {
    //     let mut parent = ProgramBuilder::default();
    //     let mut receiver_vec = vec![];
    //     let mut sender_vec = vec![];

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator2 = GeneratorContext::new(iter, sender2);
    //     parent.add_child(generator2);

    //     receiver_vec.push(receiver1);
    //     receiver_vec.push(receiver2);

        
    //     // printer contexts
    //     let (sender3, receiver3) = parent.bounded(1024);
    //     let printer1 = PrinterContext::new(receiver3);
    //     parent.add_child(printer1);

    //     let (sender4, receiver4) = parent.bounded(1024);
    //     let printer2 = PrinterContext::new(receiver4);
    //     parent.add_child(printer2);

    //     sender_vec.push(sender3);
    //     sender_vec.push(sender4);

        









    //     let (sender5, receiver5) = parent.bounded(1024);
    //     let (sender6, receiver6) = parent.bounded(1024);
        

    //     let num_tile = 100;
    //     let M = 2;
    //     let K = 2;
    //     let N = 2;
    //     let stage_dim = 6;

    //     let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(receiver_vec, 2, sender5, num_tile as usize, M as usize, K as usize, N as usize);
    //     parent.add_child(simd_pcu_adapter_upstream);

    //     let pcu = make_simd_pcu(stage_dim, receiver5, sender6);
    //     parent.add_child(pcu);

    //     let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver6, sender_vec, 2, num_tile as usize, M as usize, K as usize, N as usize);
    //     parent.add_child(simd_pcu_adapter_downstream);











    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }





















    // #[test]
    // fn test_systolic_pcu_adapter_upstream()
    // {
    //     let mut parent = ProgramBuilder::default();
    //     let mut my_vec = vec![];

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let (sender3, receiver3) = parent.bounded(1024);
    //     let (sender4, receiver4) = parent.bounded(1024);


    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator2 = GeneratorContext::new(iter, sender2);
    //     parent.add_child(generator2);

    //     my_vec.push(receiver1);
    //     my_vec.push(receiver2);

        
    //     // printer contexts
    //     let printer1 = PrinterContext::new(receiver3);
    //     parent.add_child(printer1);

    //     let printer2 = PrinterContext::new(receiver4);
    //     parent.add_child(printer2);



    //     let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(my_vec, 2, sender3, sender4, 100, 2, 1000, 2, 6, 6);
    //     parent.add_child(systolic_pcu_adapter_upstream);












    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }















    // #[test]
    // fn test_systolic_pcu_adapter_downstream()
    // {
    //     let mut parent = ProgramBuilder::default();
    //     let mut my_vec = vec![];

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let (sender3, receiver3) = parent.bounded(1024);
    //     let (sender4, receiver4) = parent.bounded(1024);

    //     let iter = || (0..13200).map(|i| (i as i32) * 1_i32);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let iter = || (0..2800).map(|i| (i as i32) * 1_i32);
    //     let generator2 = GeneratorContext::new(iter, sender2);
    //     parent.add_child(generator2);

        
    //     // printer contexts
    //     let printer1 = PrinterContext::new(receiver3);
    //     parent.add_child(printer1);

    //     let printer2 = PrinterContext::new(receiver4);
    //     parent.add_child(printer2);

        
    //     my_vec.push(sender3);
    //     my_vec.push(sender4);



    //     let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver1, receiver2, my_vec, 2, 100, 2, 2, 2, 32, 6);
    //     parent.add_child(systolic_pcu_adapter_downstream);












    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }









    // #[test]
    // fn test_systolic_pcu()
    // {
    //     let mut parent = ProgramBuilder::default();

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let (sender2, receiver2) = parent.bounded(1024);
    //     let (sender3, receiver3) = parent.bounded(1024);
    //     let (sender4, receiver4) = parent.bounded(1024);
    //     let (sender5, receiver5) = parent.bounded(1024);
    //     let (sender6, receiver6) = parent.bounded(1024);
    //     let (sender7, receiver7) = parent.bounded(1024);
    //     let (sender8, receiver8) = parent.bounded(1024);

    //     let iter = || (0..10).map(|i| (i as i32) * 1_i32);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let iter = || (0..10).map(|i| (i as i32) * 1_i32);
    //     let generator2 = GeneratorContext::new(iter, sender2);
    //     parent.add_child(generator2);

    //     let printer1 = PrinterContext::new(receiver7);
    //     parent.add_child(printer1);

    //     let printer2 = PrinterContext::new(receiver8);
    //     parent.add_child(printer2);

        
    //     let mut my_vec1 = vec![];
    //     let mut my_vec2 = vec![];
    //     my_vec1.push(receiver1);
    //     my_vec1.push(receiver2);
    //     my_vec2.push(sender7);
    //     my_vec2.push(sender8);


    //     let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(my_vec1, 2, sender3, sender4, 10, 2, 100, 2, 32, 6);
    //     parent.add_child(systolic_pcu_adapter_upstream);

    //     let pcu_lane = make_systolic_pcu(6, receiver3, sender5);
    //     parent.add_child(pcu_lane);

    //     let pcu_stage = make_systolic_pcu(32, receiver4, sender6);
    //     parent.add_child(pcu_stage);

    //     let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver5, receiver6, my_vec2, 2, 10, 2, 100, 2, 32, 6);
    //     parent.add_child(systolic_pcu_adapter_downstream);












    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }










    // #[test]
    // fn test_systolic_pcu_tmp()
    // {
    //     let mut parent = ProgramBuilder::default();

    //     // generator contexts
    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let (sender2, receiver2) = parent.bounded(1024);

    //     let iter = || (0..100).map(|i| (i as i32) * 1_i32);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let printer1 = PrinterContext::new(receiver2);
    //     parent.add_child(printer1);

        


    //     let pcu_lane = make_systolic_pcu(32, receiver1, sender2);
    //     parent.add_child(pcu_lane);











    //     // run DAM
    //     let initialized: dam::simulation::Initialized = parent
	// 	.initialize(
	// 	    InitializationOptionsBuilder::default()
	// 	        .run_flavor_inference(false)
	// 	        .build()
	// 	        .unwrap(),
	// 	)
	// 	.unwrap();
	// 	println!("{}", initialized.to_dot_string());


	// 	let executed = initialized.run(
	// 	    RunOptionsBuilder::default()
	// 	        .mode(RunMode::Simple)
	// 	        .build()
	// 	        .unwrap(),
	// 	);
	// 	println!("Elapsed cycles: {:?}", executed.elapsed_cycles());



    // }



}


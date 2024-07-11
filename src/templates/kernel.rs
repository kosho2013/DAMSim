use dam::channel::PeekResult;
use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct kernel<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Sender<usize>,
    pub latency: usize,
    pub init_inverval: usize,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> kernel<A>
where
kernel<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Sender<usize>,
        latency: usize,
        init_inverval: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let kernel = kernel {
            in_stream,
            out_stream,
            latency,
            init_inverval,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        kernel.in_stream.attach_receiver(&kernel);
        kernel.out_stream.attach_sender(&kernel);

        kernel
    }
}

impl<A: DAMType + num::Num> Context for kernel<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in1 = self.in_stream.dequeue(&self.time);

            let curr_time = self.time.tick();
            self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time + self.latency as u64, in1.unwrap().data.clone())).unwrap();
            self.time.incr_cycles(self.init_inverval as u64);
        }
    }
}





#[context_macro]
pub struct k1<A: Clone> {
    pub in_stream1: Receiver<usize>,
    pub in_stream2: Receiver<usize>,
    pub out_stream1: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> k1<A>
where
k1<A>: Context,
{
    pub fn new(
        in_stream1: Receiver<usize>,
        in_stream2: Receiver<usize>,
        out_stream1: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let k1 = k1 {
            in_stream1,
            in_stream2,
            out_stream1,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        k1.in_stream1.attach_receiver(&k1);
        k1.in_stream2.attach_receiver(&k1);
        k1.out_stream1.attach_sender(&k1);

        k1
    }
}

impl<A: DAMType + num::Num> Context for k1<A> {
    fn run(&mut self) {
        let mut cnt = 0;
        let mut flag1 = false;
        let mut flag2 = false;
        loop
        {
            if flag1 && flag2
            {
                println!("aaa");
                return;
            }

            println!("bbb, {}", cnt);
            let in1 = self.in_stream1.dequeue(&self.time);
            println!("ccc, {}", cnt);

            match in1
            {
                Ok(in1) =>
                {
                    let in1: usize = in1.data;
                    let curr_time = self.time.tick();
                    self.out_stream1.enqueue(&self.time, ChannelElement::new(curr_time+1, in1.clone())).unwrap();
                    self.time.incr_cycles(1);
                }
                Err(_) =>
                {
                    println!("ddd {}", cnt);
                    flag1 = true;
                }
            }

            println!("eee, {}", cnt);
            let in2 = self.in_stream2.dequeue(&self.time);
            println!("fff, {}", cnt);

            match in2
            {
                Ok(in2) =>
                {
                    let in2: usize = in2.data;
                }
                Err(_) => 
                {
                    println!("ggg {}", cnt);
                    flag2 = true;
                }
            }

            cnt += 1;
        }
    }
}


#[context_macro]
pub struct k2<A: Clone> {
    pub in_stream1: Receiver<usize>,
    pub out_stream1: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> k2<A>
where
k2<A>: Context,
{
    pub fn new(
        in_stream1: Receiver<usize>,
        out_stream1: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let k2 = k2 {
            in_stream1,
            out_stream1,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        k2.in_stream1.attach_receiver(&k2);
        k2.out_stream1.attach_sender(&k2);

        k2
    }
}

impl<A: DAMType + num::Num> Context for k2<A> {
    fn run(&mut self) {
        let mut cnt = 0;
        loop
        {
            println!("hhh, {}", cnt);
            let in1 = self.in_stream1.dequeue(&self.time);
            println!("iii, {}", cnt);

            match in1
            {
                Ok(in1) =>
                {
                    let in1: usize = in1.data;
                    let curr_time = self.time.tick();
                    // self.out_stream1.enqueue(&self.time, ChannelElement::new(curr_time+1, in1.clone())).unwrap();
                    self.time.incr_cycles(1);
                }
                Err(_) => 
                {   
                    println!("jjj {}", cnt);
                    return;
                }
            }
            cnt += 1;
        }
    }
}







#[context_macro]
pub struct test_kernel<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Sender<usize>,
    pub latency: usize,
    pub init_inverval: usize,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> test_kernel<A>
where
test_kernel<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Sender<usize>,
        latency: usize,
        init_inverval: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let test_kernel = test_kernel {
            in_stream,
            out_stream,
            latency,
            init_inverval,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        test_kernel.in_stream.attach_receiver(&test_kernel);
        test_kernel.out_stream.attach_sender(&test_kernel);

        test_kernel
    }
}

impl<A: DAMType + num::Num> Context for test_kernel<A> {
    fn run(&mut self) {
        for i in 0..120 {
            let peek_result = self.in_stream.peek();
            match peek_result {
                PeekResult::Something(_) =>
                {
                    println!("aaa {}", i);
                    let in1 = self.in_stream.dequeue(&self.time);
                    let curr_time = self.time.tick();
                    self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time + self.latency as u64, in1.unwrap().data.clone())).unwrap();
                    self.time.incr_cycles(self.init_inverval as u64);
                },
                PeekResult::Nothing(_) => 
                {
                    println!("bbb {}", i);
                },
                PeekResult::Closed =>
                {
                    println!("ccc {}", i);
                },                        

            } 
        }
    }
}




#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use dam::{
        shim::RunMode, simulation::{DotConvertible, InitializationOptions, InitializationOptionsBuilder, ProgramBuilder, RunOptions, RunOptionsBuilder}, templates::{ops::{ALUAddOp, ALUMulOp}, pcu::{PCUConfig, PipelineStage, PCU}}, utility_contexts::{CheckerContext, ConsumerContext, GeneratorContext, PrinterContext}
    };
    use crate::templates::kernel::test_kernel;

    use super::k1;
    use super::k2;

    #[test]
    fn test_abc()
    {
        let mut parent = ProgramBuilder::default();

        // generator contexts
        let (sender1, receiver1) = parent.bounded(1024);
        let (sender2, receiver2) = parent.bounded(1024);
        let (sender3, receiver3) = parent.bounded(1024);

        let iter = || (0..100).map(|i| (i as usize) * 1_usize);
        let gen = GeneratorContext::new(iter, sender1);
        parent.add_child(gen);

        let k1 = k1::new(receiver1, receiver2, sender3, 100, 1);
        parent.add_child(k1);

        let k2 = k2::new(receiver3, sender2, 100, 1);
        parent.add_child(k2);

        
        // run DAM
        let initialized: dam::simulation::Initialized = parent
		.initialize(
		    InitializationOptionsBuilder::default()
		        .run_flavor_inference(false)
		        .build()
		        .unwrap(),
		)
		.unwrap();
		println!("{}", initialized.to_dot_string());


		let executed = initialized.run(
		    RunOptionsBuilder::default()
		        .mode(RunMode::Simple)
		        .build()
		        .unwrap(),
		);
		println!("Elapsed cycles: {:?}", executed.elapsed_cycles());


    }












    #[test]
    fn test_xyz()
    {
        let mut parent = ProgramBuilder::default();

        // generator contexts
        let (sender1, receiver1) = parent.bounded(1024);
        let (sender2, receiver2) = parent.bounded(1024);

        let iter = || (0..100).map(|i| (i as usize) * 1_usize);
        let gen = GeneratorContext::new(iter, sender1);
        parent.add_child(gen);

        let test_kernel = test_kernel::new(receiver1, sender2, 1, 1, 100, 1);
        parent.add_child(test_kernel);

        let con = PrinterContext::new(receiver2);
        parent.add_child(con);

        
        // run DAM
        let initialized: dam::simulation::Initialized = parent
		.initialize(
		    InitializationOptionsBuilder::default()
		        .run_flavor_inference(false)
		        .build()
		        .unwrap(),
		)
		.unwrap();
		println!("{}", initialized.to_dot_string());


		let executed = initialized.run(
		    RunOptionsBuilder::default()
		        .mode(RunMode::Simple)
		        .build()
		        .unwrap(),
		);
		println!("Elapsed cycles: {:?}", executed.elapsed_cycles());


    }
}
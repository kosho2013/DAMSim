use std::collections::HashMap;

use dam::context_tools::*;
use dam::types::StaticallySized;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};


// 0: N, 1: S, 2: E, 3: W

// let mut x_diff: usize = 0;
// let mut y_diff: usize = 0;
// if self.dst_x <= self.src_x
// {
//     x_diff = self.src_x - self.dst_x;
// } else {
//     x_diff = self.dst_x - self.src_x;
// }
// if self.dst_y <= self.src_y
// {
//     y_diff = self.src_y - self.dst_y;
// } else {
//     y_diff = self.dst_y - self.src_y;
// }


// if self.dst_x < self.src_x && self.dst_y < self.src_y
// {
//     for i in 0..x_diff // N: 0
//     {
//         route.push(0);
//     }
//     for i in 0..y_diff // W: 3
//     {
//         route.push(3);
//     }
    
// } else if self.dst_x < self.src_x && self.dst_y == self.src_y
// {
//     for i in 0..x_diff // N: 0
//     {
//         route.push(0);
//     }

// } else if self.dst_x < self.src_x && self.dst_y > self.src_y
// {
//     for i in 0..x_diff // N: 0
//     {
//         route.push(0);
//     }
//     for i in 0..y_diff // E: 2
//     {
//         route.push(2);
//     }

// } else if self.dst_x == self.src_x && self.dst_y > self.src_y
// {
//     for i in 0..y_diff // E: 2
//     {
//         route.push(2);
//     }

// } else if self.dst_x > self.src_x && self.dst_y > self.src_y
// {
//     for i in 0..x_diff // S: 1
//     {
//         route.push(1);
//     }
//     for i in 0..y_diff // E: 2
//     {
//         route.push(2);
//     }

// } else if self.dst_x > self.src_x && self.dst_y == self.src_y
// {
//     for i in 0..x_diff // S: 1
//     {
//         route.push(1);
//     }

// } else if self.dst_x > self.src_x && self.dst_y < self.src_y
// {
//     for i in 0..x_diff // S: 1
//     {
//         route.push(1);
//     }
//     for i in 0..y_diff // W: 3
//     {
//         route.push(3);
//     }

// } else if self.dst_x == self.src_x && self.dst_y < self.src_y
// {
//     for i in 0..y_diff // W: 3
//     {
//         route.push(3);
//     }

// }



#[context_macro]
pub struct router_adapter_upstream<A: Clone> { // before pcu/pmu upstream adapter
    pub in_stream: Receiver<usize>,
    pub out_stream: Sender<usize>,
    pub loop_bound: usize,
    pub dst_id: usize,
    pub dummy: A,
}

impl<A: DAMType> router_adapter_upstream<A>
where
router_adapter_upstream<A>: Context,
{
    pub fn new (
        in_stream: Receiver<usize>,
        out_stream: Sender<usize>,
        loop_bound: usize,
        dst_id: usize,
        dummy: A,
    ) -> Self {
        let router_adapter_upstream = router_adapter_upstream {
            in_stream,
            out_stream,
            loop_bound,
            dst_id,
            dummy,
            context_info: Default::default(),
        };

        router_adapter_upstream.in_stream.attach_receiver(&router_adapter_upstream);
        router_adapter_upstream.out_stream.attach_sender(&router_adapter_upstream);

        router_adapter_upstream
    }
}

impl<A: DAMType + num::Num> Context for router_adapter_upstream<A> {
    fn run(&mut self) {
        for _ in 0..self.loop_bound {
            let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

            let curr_time = self.time.tick();
            self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time+1, self.dst_id)).unwrap();
            self.time.incr_cycles(1);
        }
    }
}













#[context_macro]
pub struct router_adapter_downstream<A: Clone> { // after pcu/pmu upstream adapter
    pub in_stream: Receiver<usize>,
    pub out_stream: Sender<usize>,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> router_adapter_downstream<A>
where
router_adapter_downstream<A>: Context,
{
    pub fn new (
        in_stream: Receiver<usize>,
        out_stream: Sender<usize>,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let router_adapter_downstream = router_adapter_downstream {
            in_stream,
            out_stream,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };

        router_adapter_downstream.in_stream.attach_receiver(&router_adapter_downstream);
        router_adapter_downstream.out_stream.attach_sender(&router_adapter_downstream);

        router_adapter_downstream
    }
}

impl<A: DAMType + num::Num> Context for router_adapter_downstream<A> {
    fn run(&mut self) {
        for i in 0..self.loop_bound {
            let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

            let curr_time = self.time.tick();
            self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time+1, in_data.clone())).unwrap();
            self.time.incr_cycles(1);
        }
    }
}



#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dam::{
        shim::RunMode, simulation::{DotConvertible, InitializationOptions, InitializationOptionsBuilder, ProgramBuilder, RunOptions, RunOptionsBuilder}, templates::{ops::{ALUAddOp, ALUMulOp}, pcu::{PCUConfig, PipelineStage, PCU}}, utility_contexts::{CheckerContext, GeneratorContext, PrinterContext}
    };

    use crate::templates::{my_pcu::{make_simd_pcu, make_systolic_pcu}, router_adapter::router_adapter_upstream, pcu_adapter::{simd_pcu_adapter_downstream, simd_pcu_adapter_upstream, systolic_pcu_adapter_downstream, systolic_pcu_adapter_upstream}};

    // #[test]
    // fn test_router_adapter_upstream()
    // {
    //     let dummy = 1;
    //     let mut parent = ProgramBuilder::default();



    //     let (sender1, receiver1) = parent.bounded(1024);
    //     let (sender2, receiver2) = parent.bounded(1024);

    //     let iter = || (0..100).map(|i| (i as usize) * 1_usize);
    //     let generator1 = GeneratorContext::new(iter, sender1);
    //     parent.add_child(generator1);

    //     let router_adapter_upstream = router_adapter_upstream::new(receiver1, sender2, 100, 10, 12, 32, 40, dummy);
    //     parent.add_child(router_adapter_upstream);

    //     let printer1 = PrinterContext::new(receiver2);
    //     parent.add_child(printer1);



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

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










#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dam::shim::RunMode;
    use dam::simulation::{DotConvertible, InitializationOptions, InitializationOptionsBuilder, ProgramBuilder, RunOptions, RunOptionsBuilder};

    use dam::{templates::ops::ALUAddOp, utility_contexts::*};

    use crate::templates::primitive::{ALUExpOp, Exp, Token};
    use crate::templates::router_mesh::router_mesh;
    use crate::token_vec;

    #[test]
    fn test_router_mesh() {
        let mut parent = ProgramBuilder::default();
        


        let (sender1, receiver1) = parent.bounded(1024);
        let (sender2, receiver2) = parent.bounded(1024);
        let (sender3, receiver3) = parent.bounded(1024);
        let (sender4, receiver4) = parent.bounded(1024);
        let (sender5, receiver5) = parent.bounded(1024);



        let iter = || (0..(1000)).map(|i| (i as usize) * 0_usize);
        let gen1 = GeneratorContext::new(iter, sender1);
		parent.add_child(gen1);

        let iter = || (0..(1000)).map(|i| (i as usize) * 0_usize);
        let gen2 = GeneratorContext::new(iter, sender2);
		parent.add_child(gen2);

        let iter = || (0..(1000)).map(|i| (i as usize) * 0_usize);
        let gen3 = GeneratorContext::new(iter, sender3);
		parent.add_child(gen3);

        let iter = || (0..(1000)).map(|i| (i as usize) * 0_usize);
        let gen4 = GeneratorContext::new(iter, sender4);
		parent.add_child(gen4);




        let mut in_stream = vec![];
        in_stream.push(receiver1);
        in_stream.push(receiver2);
        in_stream.push(receiver3);
        in_stream.push(receiver4);

        let mut out_stream = vec![];
        out_stream.push(sender5);

        let in_len = 4;
        let mut in_dict = HashMap::new();
        in_dict.insert("N_in".to_owned(), (0, 1));
        in_dict.insert("S_in".to_owned(), (1, 1));
        in_dict.insert("E_in".to_owned(), (2, 1));
        in_dict.insert("W_in".to_owned(), (3, 1));

        let mut out_dict = HashMap::new();
        out_dict.insert("L_out".to_owned(), (0, 4));
        let out_len = 1;


        let x_dim = 1;
        let y_dim = 1;
        let x = 0;
        let y = 0;
        let num_input = 1000;
        let num_vc = 10;
        let dummy = 0;


        let router_mesh = router_mesh::new(in_stream, in_dict, in_len, out_stream, out_dict, out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
        parent.add_child(router_mesh);



        let con1 = PrinterContext::new(receiver5);
		parent.add_child(con1);
        

        

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
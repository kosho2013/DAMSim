use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};


#[context_macro]
pub struct pmu_adapter_upstream<A: Clone> {
    pub in_stream: Vec<Receiver<A>>,
    pub in_len: usize,
    pub out_stream_wr_addr: Sender<usize>,
    pub out_stream_wr_data: Sender<A>,
    pub loop_bound: usize,
    pub counter: usize,
}

impl<A: DAMType> pmu_adapter_upstream<A>
where
pmu_adapter_upstream<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<A>>,
        in_len: usize,
        out_stream_wr_addr: Sender<usize>,
        out_stream_wr_data: Sender<A>,
        loop_bound: usize,
        counter: usize,
    ) -> Self {
        let pmu_adapter_upstream = pmu_adapter_upstream {
            in_stream,
            in_len,
            out_stream_wr_addr,
            out_stream_wr_data,
            loop_bound,
            counter,
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
        for _ in 0..self.loop_bound {
            let mut in_vec = vec![];

            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                in_vec.push(self.in_stream[idx].dequeue(&self.time));
            }

            let in_data = in_vec.remove(0).unwrap().data;
            let out_data: A;
            out_data = in_data.clone();

            for _ in 0..self.counter
            {
                let curr_time = self.time.tick();
                self.out_stream_wr_addr.enqueue(&self.time, ChannelElement::new(curr_time + 1, 0)).unwrap();
                self.out_stream_wr_data.enqueue(&self.time, ChannelElement::new(curr_time + 1, out_data.clone())).unwrap();
                self.time.incr_cycles(1);
                cnt += 1;
            }
        }
    }
}








#[context_macro]
pub struct pmu_adapter_downstream<A: Clone> {
    pub in_stream: Receiver<A>,
    pub out_stream: Vec<Sender<A>>,
    pub out_len: usize,
    pub loop_bound: usize,
    pub counter: usize,
}

impl<A: DAMType> pmu_adapter_downstream<A>
where
pmu_adapter_downstream<A>: Context,
{
    pub fn new(
        in_stream: Receiver<A>,
        out_stream: Vec<Sender<A>>,
        out_len: usize,
        loop_bound: usize,
        counter: usize,
    ) -> Self {
        let pmu_adapter_downstream = pmu_adapter_downstream {
            in_stream,
            out_stream,
            out_len,
            loop_bound,
            counter,
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
        let tmp = self.counter * self.loop_bound; 
        for i in 0..tmp
        {
            let in1: Result<ChannelElement<A>, dam::channel::DequeueError> = self.in_stream.dequeue(&self.time);
            let mut data = in1.unwrap().data.clone();

            if (i % self.counter == 0)
            {
                for j in 0..self.out_len
                {
                    let curr_time = self.time.tick();
                    let idx: usize = j.try_into().unwrap();
                    self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time + 1, data.clone())).unwrap();
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
        channel::ChannelElement, shim::RunMode, simulation::{DotConvertible, InitializationOptions, InitializationOptionsBuilder, ProgramBuilder, RunOptions, RunOptionsBuilder}, templates::{datastore::Behavior, ops::{ALUAddOp, ALUMulOp}, pcu::{PCUConfig, PipelineStage, PCU}, pmu::{PMUReadBundle, PMUWriteBundle, PMU}}, utility_contexts::{CheckerContext, ConsumerContext, FunctionContext, GeneratorContext, PrinterContext}
    };

    use crate::templates::{my_pcu::{make_simd_pcu, make_systolic_pcu}, pcu_adapter::{simd_pcu_adapter_downstream, simd_pcu_adapter_upstream, systolic_pcu_adapter_downstream, systolic_pcu_adapter_upstream}, pmu_adapter::{pmu_adapter_downstream, pmu_adapter_upstream}};





    #[test]
    fn test_pmu()
    {
        let mut parent = ProgramBuilder::default();

        let cycle = 10000;
        let capacity = 50;
        let loop_bound = 5;

        let (sender1, receiver1) = parent.bounded(1024);
        let (sender2, receiver2) = parent.bounded(1024);
        let (sender3, receiver3) = parent.bounded(1024);
        let (sender4, receiver4) = parent.bounded(1024);

        let (wr_addr_sender, wr_addr_receiver) = parent.bounded(1024);
        let (wr_data_sender, wr_data_receiver) = parent.bounded(1024);
        let (ack_sender, ack_receiver) = parent.bounded(1024);
        let (rd_addr_sender, rd_addr_receiver) = parent.bounded(1024);
        let (rd_data_sender, rd_data_receiver) = parent.bounded(1024);


        let mut my_vec1 = vec![];
        my_vec1.push(receiver1);
        my_vec1.push(receiver2);

        let mut my_vec2 = vec![];
        my_vec2.push(sender3);
        my_vec2.push(sender4);



        let generator1 = GeneratorContext::new(
            move || (0..loop_bound).map(|x| (x as usize) * 1_usize),
            sender1,
        );
        parent.add_child(generator1);

        let generator2 = GeneratorContext::new(
            move || (0..loop_bound).map(|x| (x as usize) * 1_usize),
            sender2,
        );
        parent.add_child(generator2);





        let pmu_adapter_upstream = pmu_adapter_upstream::new(my_vec1, 2, wr_addr_sender, wr_data_sender, loop_bound, cycle);
        parent.add_child(pmu_adapter_upstream);



        let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
            capacity,
            Behavior {
                mod_address: false,
                use_default_value: false,
            },
        );
        pmu.add_writer(PMUWriteBundle {
            addr: wr_addr_receiver,
            data: wr_data_receiver,
            ack: ack_sender,
        });
        pmu.add_reader(PMUReadBundle {
            addr: rd_addr_receiver,
            resp: rd_data_sender,
        });
        parent.add_child(pmu);



        
        let mut rd_addr_gen = FunctionContext::new();
        ack_receiver.attach_receiver(&rd_addr_gen);
        rd_addr_sender.attach_sender(&rd_addr_gen);
        let tmp = cycle * loop_bound;
        rd_addr_gen.set_run(move |time| {
            for idx in 0..tmp
            {
                ack_receiver.dequeue(time).unwrap();
                let curr_time = time.tick();
                rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
            }
        });
        parent.add_child(rd_addr_gen);




        
        let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, my_vec2, 2, loop_bound, cycle);
        parent.add_child(pmu_adapter_downstream);




        let printer1 = PrinterContext::new(receiver3);
        parent.add_child(printer1);

        let printer2 = PrinterContext::new(receiver4);
        parent.add_child(printer2);









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


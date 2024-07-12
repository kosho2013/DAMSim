use std::collections::HashMap;

use dam::channel::PeekResult;
use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct router<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_dict: HashMap<String, usize>,
    pub in_len: usize,
    pub out_stream: Vec<Sender<usize>>,
    pub out_dict: HashMap<String, usize>,
    pub out_len: usize,
    pub x_dim: usize,
    pub y_dim: usize,
    pub x: usize,
    pub y: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router<A>
where
router<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_dict: HashMap<String, usize>,
        in_len: usize,
        out_stream: Vec<Sender<usize>>,
        out_dict: HashMap<String, usize>,
        out_len: usize,
        x_dim: usize,
        y_dim: usize,
        x: usize,
        y: usize,
        dummy: A,
    ) -> Self {
        let router = router {
            in_stream,
            in_dict,
            in_len,
            out_stream,
            out_dict,
            out_len,
            x_dim,
            y_dim,
            x,
            y,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            router.in_stream[idx].attach_receiver(&router);
        }
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            router.out_stream[idx].attach_sender(&router);
        }

        router
    }
}

impl<A: DAMType + num::Num> Context for router<A> {
    fn run(&mut self) {

        let invalid = 999999;

        let mut in_idx_vec = vec![]; // NSEWL
        let mut in_closed = vec![]; // NSEWL
        for _ in 0..5
        {
            in_idx_vec.push(invalid);
            in_closed.push(false);
        }

        if self.in_dict.contains_key("N_in")
        {
            in_idx_vec[0] = self.in_dict["N_in"];
        } else {
            in_closed[0] = true;
        }
        if self.in_dict.contains_key("S_in")
        {
            in_idx_vec[1] = self.in_dict["S_in"];
        } else {
            in_closed[1] = true;
        }
        if self.in_dict.contains_key("E_in")
        {
            in_idx_vec[2] = self.in_dict["E_in"];
        } else {
            in_closed[2] = true;
        }
        if self.in_dict.contains_key("W_in")
        {
            in_idx_vec[3] = self.in_dict["W_in"];
        } else {
            in_closed[3] = true;
        }
        if self.in_dict.contains_key("L_in")
        {
            in_idx_vec[4] = self.in_dict["L_in"];
        } else {
            in_closed[4] = true;
        }

        let mut out_idx_vec = vec![]; // NSEWL
        for _ in 0..5
        {
            out_idx_vec.push(invalid);
        }

        if self.out_dict.contains_key("N_out")
        {
            out_idx_vec[0] = self.out_dict["N_out"];
        }
        if self.out_dict.contains_key("S_out")
        {
            out_idx_vec[1] = self.out_dict["S_out"];
        }
        if self.out_dict.contains_key("E_out")
        {
            out_idx_vec[2] = self.out_dict["E_out"];
        }
        if self.out_dict.contains_key("W_out")
        {
            out_idx_vec[3] = self.out_dict["W_out"];
        }
        if self.out_dict.contains_key("L_out")
        {
            out_idx_vec[4] = self.out_dict["L_out"];
        }

        
        let mut cnt = 0;


        loop
        {
            let mut tmp: usize = 0;
            for ele in &in_closed
            {
                if *ele == true
                {
                    tmp += 1;
                }
            }
            if tmp == 5
            {
                return;
            }

            // read from all input ports
            let mut data_vec = vec![];
            let mut dst_x_vec = vec![];
            let mut dst_y_vec = vec![];

            for i in 0..5 
            {
                if in_closed[i]
                {
                    data_vec.push(invalid);
                    dst_x_vec.push(invalid);
                    dst_y_vec.push(invalid);
                } else
                {
                    let peek_result = self.in_stream[in_idx_vec[i]].peek();
                    match peek_result {
                        PeekResult::Something(_) =>
                        {
                            let data = self.in_stream[in_idx_vec[i]].dequeue(&self.time).unwrap().data;
                            let dst_x = data / self.y_dim;
                            let dst_y = data % self.y_dim;
                            data_vec.push(data);
                            dst_x_vec.push(dst_x);
                            dst_y_vec.push(dst_y);
                        },
                        PeekResult::Nothing(_) => 
                        {
                            data_vec.push(invalid);
                            dst_x_vec.push(invalid);
                            dst_y_vec.push(invalid);
                        },
                        PeekResult::Closed =>
                        {
                            data_vec.push(invalid);
                            dst_x_vec.push(invalid);
                            dst_y_vec.push(invalid);
                            in_closed[i] = true;
                        },
                    }
                }
            }

            // if cnt <= 1000 && self.x == 1 && self.y == 1
            // {
            //     println!("x:{}, y:{}, in_idx_vec{:?}, data_vec{:?}, dst_x_vec{:?}, dst_y_vec{:?}, in_closed{:?}", self.x, self.y, in_idx_vec, data_vec, dst_x_vec, dst_y_vec, in_closed);
            // }

            for i in 0..5
            {
                if data_vec[i] != invalid && dst_x_vec[i] != invalid && dst_y_vec[i] != invalid
                { 
                    if dst_x_vec[i] == self.x && dst_y_vec[i] == self.y // exit local port
                    {
                        if out_idx_vec[4] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[4]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] == self.x && dst_y_vec[i] < self.y // exit W port
                    {
                        if out_idx_vec[3] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[3]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] < self.x && dst_y_vec[i] < self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] < self.x && dst_y_vec[i] == self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] < self.x && dst_y_vec[i] > self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] == self.x && dst_y_vec[i] > self.y // exit E port
                    {
                        if out_idx_vec[2] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[2]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] > self.x && dst_y_vec[i] > self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] > self.x && dst_y_vec[i] == self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }

                    } else if dst_x_vec[i] > self.x && dst_y_vec[i] < self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                        }
                        
                    } else
                    {
                        panic!("Wrong!");
                    }
                }
            }
            self.time.incr_cycles(1);


            cnt += 1;
        }
    }
}












#[cfg(test)]
mod tests {
    use std::{collections::HashMap, hash::Hash};
    use dam::{
        shim::RunMode, simulation::{DotConvertible, InitializationOptions, InitializationOptionsBuilder, ProgramBuilder, RunOptions, RunOptionsBuilder}, templates::{ops::{ALUAddOp, ALUMulOp}, pcu::{PCUConfig, PipelineStage, PCU}}, utility_contexts::{CheckerContext, ConsumerContext, GeneratorContext, PrinterContext}
    };
    use crate::templates::{kernel::test_kernel, router::router};

    #[test]
    fn test_zzzz()
    {
        let mut parent = ProgramBuilder::default();

        // generator contexts
        let (sender1, receiver1) = parent.bounded(1);
        let (sender2, receiver2) = parent.bounded(1);
        let (sender3, receiver3) = parent.bounded(1);
        let (sender4, receiver4) = parent.bounded(1);
        let (sender5, receiver5) = parent.bounded(1);
        let (sender6, receiver6) = parent.bounded(1);
        let (sender7, receiver7) = parent.bounded(1);
        let (sender8, receiver8) = parent.bounded(1);
        let (sender9, receiver9) = parent.bounded(1);
        let (sender10, receiver10) = parent.bounded(1);

        let iter = || (0..1000).map(|i: usize| (3 as usize) * 1_usize);
        let gen_N = GeneratorContext::new(iter, sender1);
        parent.add_child(gen_N);

        let iter = || (0..1000).map(|i: usize| (3 as usize) * 1_usize);
        let gen_S = GeneratorContext::new(iter, sender2);
        parent.add_child(gen_S);

        let iter = || (0..1000).map(|i: usize| (3 as usize) * 1_usize);
        let gen_E = GeneratorContext::new(iter, sender3);
        parent.add_child(gen_E);

        let iter = || (0..1000).map(|i: usize| (3 as usize) * 1_usize);
        let gen_W = GeneratorContext::new(iter, sender4);
        parent.add_child(gen_W);

        let iter = || (0..1000).map(|i: usize| (3 as usize) * 1_usize);
        let gen_L = GeneratorContext::new(iter, sender5);
        parent.add_child(gen_L);


        let mut receiver_vec = vec![];
        receiver_vec.push(receiver1);
        receiver_vec.push(receiver2);
        receiver_vec.push(receiver3);
        receiver_vec.push(receiver4);
        receiver_vec.push(receiver5);

        let mut in_dict: HashMap<String, usize> = HashMap::new();
        in_dict.insert("N_in".to_owned(), 0);
        in_dict.insert("S_in".to_owned(), 1);
        in_dict.insert("E_in".to_owned(), 2);
        in_dict.insert("W_in".to_owned(), 3);
        in_dict.insert("L_in".to_owned(), 4);

        let in_len = 5;

        let mut sender_vec = vec![];
        sender_vec.push(sender6);
        sender_vec.push(sender7);
        sender_vec.push(sender8);
        sender_vec.push(sender9);
        sender_vec.push(sender10);

        let mut out_dict: HashMap<String, usize> = HashMap::new();
        out_dict.insert("N_out".to_owned(), 0);
        out_dict.insert("S_out".to_owned(), 1);
        out_dict.insert("E_out".to_owned(), 2);
        out_dict.insert("W_out".to_owned(), 3);
        out_dict.insert("L_out".to_owned(), 4);

        let out_len = 5;

        let dummy = 1;
        let router = router::new(receiver_vec, in_dict, in_len, sender_vec, out_dict, out_len, 3, 3, 1, 2, dummy);
        parent.add_child(router);


        let p_N = ConsumerContext::new(receiver6);
        parent.add_child(p_N);
        let p_S = ConsumerContext::new(receiver7);
        parent.add_child(p_S);
        let p_E = ConsumerContext::new(receiver8);
        parent.add_child(p_E);
        let p_W = ConsumerContext::new(receiver9);
        parent.add_child(p_W);
        let p_L = ConsumerContext::new(receiver10);
        parent.add_child(p_L);



        
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
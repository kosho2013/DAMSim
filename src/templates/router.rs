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
    pub loop_bound: usize,
    pub x_dim: usize,
    pub y_dim: usize,
    pub x: usize,
    pub y: usize,
    pub num_vc: usize,
    pub buffer_depth: usize,
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
        loop_bound: usize,
        x_dim: usize,
        y_dim: usize,
        x: usize,
        y: usize,
        num_vc: usize,
        buffer_depth: usize, 
        dummy: A,
    ) -> Self {
        let router = router {
            in_stream,
            in_dict,
            in_len,
            out_stream,
            out_dict,
            out_len,
            loop_bound,
            dummy,
            x_dim,
            y_dim,
            x,
            y,
            num_vc,
            buffer_depth,
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
        let mut in_closed_vec = vec![]; // NSEWL

        for i in 0..5
        {
            in_idx_vec.push(invalid);
            in_closed_vec.push(false);
        }

        if self.in_dict.contains_key("N_in")
        {
            in_idx_vec[0] = self.in_dict["N_in"];
        } else {
            in_closed_vec[0] = true;
        }
        if self.in_dict.contains_key("S_in")
        {
            in_idx_vec[1] = self.in_dict["S_in"];
        } else {
            in_closed_vec[1] = true;
        }
        if self.in_dict.contains_key("E_in")
        {
            in_idx_vec[2] = self.in_dict["E_in"];
        } else {
            in_closed_vec[2] = true;
        }
        if self.in_dict.contains_key("W_in")
        {
            in_idx_vec[3] = self.in_dict["W_in"];
        } else {
            in_closed_vec[3] = true;
        }
        if self.in_dict.contains_key("L_in")
        {
            in_idx_vec[4] = self.in_dict["L_in"];
        } else {
            in_closed_vec[4] = true;
        }

        let mut out_idx_vec = vec![]; // NSEWL
        if self.in_dict.contains_key("N_out")
        {
            out_idx_vec[0] = self.in_dict["N_out"];
        }
        if self.in_dict.contains_key("S_out")
        {
            out_idx_vec[1] = self.in_dict["S_out"];
        }
        if self.in_dict.contains_key("E_out")
        {
            out_idx_vec[2] = self.in_dict["E_out"];
        }
        if self.in_dict.contains_key("W_out")
        {
            out_idx_vec[3] = self.in_dict["W_out"];
        }
        if self.in_dict.contains_key("L_out")
        {
            out_idx_vec[4] = self.in_dict["L_out"];
        }


        
        loop
        {
            let mut tmp: usize = 0;
            for ele in &in_closed_vec
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


            while true {
                let mut tmp: usize = 0;
                for ele in &in_closed_vec
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

                let mut at_least_one_dequeued = false;
                for i in 0..5 
                {
                    if !in_closed_vec[i]
                    {
                        let peek_result = self.in_stream[in_idx_vec[i]].peek();
                        match peek_result {
                            PeekResult::Something(_) =>
                            {
                                // println!("x:{}, y:{}, i:{}, aaaaa", self.x, self.y, i);

                                let data = self.in_stream[in_idx_vec[i]].dequeue(&self.time).unwrap().data;
                                let dst_x = data / self.y_dim;
                                let dst_y = data % self.y_dim;

                                // println!("x:{}, y:{}, data{}, dst_x{}, dst_y{}", self.x, self.y, data, dst_x, dst_y);
                                
                                data_vec.push(data);
                                dst_x_vec.push(dst_x);
                                dst_y_vec.push(dst_y);

                                at_least_one_dequeued = true;
                            },
                            PeekResult::Nothing(_) => 
                            {
                                // println!("x:{}, y:{}, i:{}, bbbbb", self.x, self.y, i);
                            },
                            PeekResult::Closed =>
                            {
                                // println!("x:{}, y:{}, i:{}, ccccc", self.x, self.y, i);
                                in_closed_vec[i] = true;
                                data_vec.push(invalid);
                                dst_x_vec.push(invalid);
                                dst_y_vec.push(invalid);
                            },                        

                        }
                    }
                }

                if at_least_one_dequeued
                {
                    break;
                }
            }
            
                

            println!("x:{}, y:{}, in_closed_vec{:?}, data_vec{:?}, dst_x_vec{:?}, dst_y_vec{:?}", self.x, self.y, in_closed_vec, data_vec, dst_x_vec, dst_y_vec);


            for i in 0..5 
            {
                if !in_closed_vec[i]
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


        }

    }
}
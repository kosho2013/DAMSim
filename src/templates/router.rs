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
    pub in_dict: HashMap<String, (usize, usize)>,
    pub in_len: usize,
    pub out_stream: Vec<Sender<usize>>,
    pub out_dict: HashMap<String, (usize, usize)>,
    pub out_len: usize,
    pub x_dim: usize,
    pub y_dim: usize,
    pub x: usize,
    pub y: usize,
    pub num_input: usize,
    pub dummy: A,
}

impl<A: DAMType + num::Num> router<A>
where
router<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_dict: HashMap<String, (usize, usize)>,
        in_len: usize,
        out_stream: Vec<Sender<usize>>,
        out_dict: HashMap<String, (usize, usize)>,
        out_len: usize,
        x_dim: usize,
        y_dim: usize,
        x: usize,
        y: usize,
        num_input: usize,
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
            num_input,
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
        let mut in_invalid = vec![]; // NSEWL
        let mut in_num_received_limit = vec![]; // NSEWL
        let mut in_num_recieved = [0, 0, 0, 0, 0]; // NSEWL

        for _ in 0..5
        {
            in_idx_vec.push(invalid);
            in_invalid.push(false);
            in_num_received_limit.push(invalid);
        }

        if self.in_dict.contains_key("N_in")
        {
            in_idx_vec[0] = self.in_dict["N_in"].0;
            in_num_received_limit[0] = self.in_dict["N_in"].1;
        } else {
            in_invalid[0] = true;
        }
        if self.in_dict.contains_key("S_in")
        {
            in_idx_vec[1] = self.in_dict["S_in"].0;
            in_num_received_limit[1] = self.in_dict["S_in"].1;
        } else {
            in_invalid[1] = true;
        }
        if self.in_dict.contains_key("E_in")
        {
            in_idx_vec[2] = self.in_dict["E_in"].0;
            in_num_received_limit[2] = self.in_dict["E_in"].1;
        } else {
            in_invalid[2] = true;
        }
        if self.in_dict.contains_key("W_in")
        {
            in_idx_vec[3] = self.in_dict["W_in"].0;
            in_num_received_limit[3] = self.in_dict["W_in"].1;
        } else {
            in_invalid[3] = true;
        }
        if self.in_dict.contains_key("L_in")
        {
            in_idx_vec[4] = self.in_dict["L_in"].0;
            in_num_received_limit[4] = self.in_dict["L_in"].1;
        } else {
            in_invalid[4] = true;
        }




        let mut out_idx_vec = vec![]; // NSEWL
        let mut out_invalid = vec![]; // NSEWL
        let mut out_num_sent_limit = vec![]; // NSEWL
        let mut out_num_sent = [0, 0, 0, 0, 0]; // NSEWL

        for _ in 0..5
        {
            out_idx_vec.push(invalid);
            out_invalid.push(false);
            out_num_sent_limit.push(invalid);
        }

        if self.out_dict.contains_key("N_out")
        {
            out_idx_vec[0] = self.out_dict["N_out"].0;
            out_num_sent_limit[0] = self.out_dict["N_out"].1;
        } else {
            out_invalid[0] = true;
        }
        if self.out_dict.contains_key("S_out")
        {
            out_idx_vec[1] = self.out_dict["S_out"].0;
            out_num_sent_limit[1] = self.out_dict["S_out"].1;
        } else {
            out_invalid[1] = true;
        }
        if self.out_dict.contains_key("E_out")
        {
            out_idx_vec[2] = self.out_dict["E_out"].0;
            out_num_sent_limit[2] = self.out_dict["E_out"].1;
        } else {
            out_invalid[2] = true;
        }
        if self.out_dict.contains_key("W_out")
        {
            out_idx_vec[3] = self.out_dict["W_out"].0;
            out_num_sent_limit[3] = self.out_dict["W_out"].1;
        } else {
            out_invalid[3] = true;
        }
        if self.out_dict.contains_key("L_out")
        {
            out_idx_vec[4] = self.out_dict["L_out"].0;
            out_num_sent_limit[4] = self.out_dict["L_out"].1;
        } else {
            out_invalid[4] = true;
        }


        
        

        // println!("x:{}, y:{}, in_idx_vec{:?}, in_num_received_limit{:?}, out_idx_vec{:?}, out_num_sent_limit{:?}   ", self.x, self.y, in_idx_vec, in_num_received_limit, out_idx_vec, out_num_sent_limit);






        loop
        {
            // read from all input ports
            let mut data_vec = vec![];
            let mut dst_x_vec = vec![];
            let mut dst_y_vec = vec![];

            for i in 0..5 // NSEWL
            {
                if in_invalid[i]
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

                            in_num_recieved[i] += 1;
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
                            in_invalid[i] = true;
                        },
                    }
                }
            }



            

            for i in 0..5 // NSEWL
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
                            out_num_sent[4] += 1;
                        }

                    } else if dst_x_vec[i] == self.x && dst_y_vec[i] < self.y // exit W port
                    {
                        if out_idx_vec[3] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[3]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[3] += 1;
                        }

                    } else if dst_x_vec[i] < self.x && dst_y_vec[i] < self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[0] += 1;
                        }

                    } else if dst_x_vec[i] < self.x && dst_y_vec[i] == self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[0] += 1;
                        }

                    } else if dst_x_vec[i] < self.x && dst_y_vec[i] > self.y // exit N port
                    {
                        if out_idx_vec[0] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[0]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[0] += 1;
                        }

                    } else if dst_x_vec[i] == self.x && dst_y_vec[i] > self.y // exit E port
                    {
                        if out_idx_vec[2] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[2]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[2] += 1;
                        }

                    } else if dst_x_vec[i] > self.x && dst_y_vec[i] > self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[1] += 1;
                        }

                    } else if dst_x_vec[i] > self.x && dst_y_vec[i] == self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[1] += 1;
                        }

                    } else if dst_x_vec[i] > self.x && dst_y_vec[i] < self.y // exit S port
                    {
                        if out_idx_vec[1] == invalid
                        {
                            panic!("Wrong!");
                        } else {
                            let curr_time = self.time.tick();
                            self.out_stream[out_idx_vec[1]].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, data_vec[i].clone())).unwrap();
                            out_num_sent[1] += 1;
                        }
                        
                    } else
                    {
                        panic!("Wrong!");
                    }
                }
            }
            self.time.incr_cycles(1);






            




            // return when all input ports are closed
            // let mut tmp: usize = 0;
            // for ele in &in_invalid
            // {
            //     if *ele == true
            //     {
            //         tmp += 1;
            //     }
            // }
            // if tmp == 5
            // {
            //     return;
            // }


            // return when all input ports have received all packets and all output ports have sent all packets
            let mut cnt1 = 0;
            for i in 0..5
            {
                if in_invalid[i]
                {
                    cnt1 += 1;
                } else {
                    if in_num_recieved[i] == in_num_received_limit[i] * self.num_input
                    {
                        cnt1 += 1;
                    }
                }
            }

            let mut cnt2 = 0;
            for i in 0..5
            {
                if out_invalid[i]
                {
                    cnt2 += 1;
                } else {
                    if out_num_sent[i] == out_num_sent_limit[i] * self.num_input
                    {
                        cnt2 += 1;
                    }
                }
            }

            if cnt1 == 5 && cnt2 == 5
            {
                println!("finished!!!!!!!!!!!!!!!!!!!!!!! {}, {}", self.x, self.y);
                return;
            }

        }
    }
}
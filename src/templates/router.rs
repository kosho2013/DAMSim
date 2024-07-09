use std::collections::HashMap;

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

        let mut N_in_packet_idx = invalid;
        let mut S_in_packet_idx = invalid;
        let mut E_in_packet_idx = invalid;
        let mut W_in_packet_idx = invalid;
        let mut L_in_packet_idx = invalid;

        if self.in_dict.contains_key("N_in_packet")
        {
            N_in_packet_idx = self.in_dict["N_in_packet"];
        }
        if self.in_dict.contains_key("S_in_packet")
        {
            S_in_packet_idx = self.in_dict["S_in_packet"];
        }
        if self.in_dict.contains_key("E_in_packet")
        {
            E_in_packet_idx = self.in_dict["E_in_packet"];
        }
        if self.in_dict.contains_key("W_in_packet")
        {
            W_in_packet_idx = self.in_dict["W_in_packet"];
        }
        if self.in_dict.contains_key("L_in_packet")
        {
            L_in_packet_idx = self.in_dict["L_in_packet"];
        }

        let mut N_out_packet_idx = invalid;
        let mut S_out_packet_idx = invalid;
        let mut E_out_packet_idx = invalid;
        let mut W_out_packet_idx = invalid;
        let mut L_out_packet_idx = invalid;

        if self.in_dict.contains_key("N_out_packet")
        {
            N_out_packet_idx = self.in_dict["N_out_packet"];
        }
        if self.in_dict.contains_key("S_out_packet")
        {
            S_out_packet_idx = self.in_dict["S_out_packet"];
        }
        if self.in_dict.contains_key("E_out_packet")
        {
            E_out_packet_idx = self.in_dict["E_out_packet"];
        }
        if self.in_dict.contains_key("W_out_packet")
        {
            W_out_packet_idx = self.in_dict["W_out_packet"];
        }
        if self.in_dict.contains_key("L_out_packet")
        {
            L_out_packet_idx = self.in_dict["L_out_packet"];
        }

        loop
        {
            for j in 0..self.in_len
            {
                let in_data = self.in_stream[j].dequeue(&self.time).unwrap().data;

                match in_data {
                    Ok(value) => println!("Success: The result is {}", value),
                    Err(error) => println!("Error: {}", error),
                }

                let dst_x = in_packet / self.y_dim;
                let dst_y = in_packet % self.y_dim;

                buffer.push(in_packet);


                // receive credit
                // for k in 0..self.in_packet_len-1
                // {
                //     let in_credit = self.in_credit[k].dequeue(&self.time).unwrap().data;
                // }

                // send credit
                // for k in 0..self.in_packet_len-1
                // {
                //     let curr_time: dam::structures::Time = self.time.tick();
                //     self.out_credit[k].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, buffer.len().clone())).unwrap();
                //     self.time.incr_cycles(1);
                // }



                if dst_x == self.x && dst_y == self.y // exit local port
                {
                    if out_L_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_L_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x == self.x && dst_y < self.y // exit W port
                {
                    if out_W_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_W_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x < self.x && dst_y < self.y // exit N port
                {
                    if out_N_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_N_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x < self.x && dst_y == self.y // exit N port
                {
                    if out_N_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_N_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x < self.x && dst_y > self.y // exit N port
                {
                    if out_N_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_N_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }


                } else if dst_x == self.x && dst_y > self.y // exit E port
                {
                    if out_E_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_E_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x > self.x && dst_y > self.y // exit S port
                {
                    if out_S_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_S_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x > self.x && dst_y == self.y // exit S port
                {
                    if out_S_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_S_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x > self.x && dst_y < self.y // exit S port
                {
                    if out_S_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_packet[out_S_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, in_packet.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }
                    
                } else
                {
                    panic!("Wrong!");
                }



                

            }
        }

    }
}
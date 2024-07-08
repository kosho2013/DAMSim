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
    pub in_len: usize,
    pub in_direction: Vec<String>,
    // pub in_credit: Vec<Receiver<usize>>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub out_direction: Vec<String>,
    // pub out_credit: Vec<Sender<usize>>,
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
        in_len: usize,
        in_direction: Vec<String>,
        // in_credit: Vec<Receiver<usize>>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        out_direction: Vec<String>,
        // out_credit: Vec<Sender<usize>>,
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
            in_len,
            in_direction,
            // in_credit,
            out_stream,
            out_len,
            out_direction,
            // out_credit,
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

        let mut buffer = vec![];

        let invalid = 999999;

        let mut in_N_idx = invalid;
        for i in 0..self.in_direction.len()
        {
            if self.in_direction[i] == "N"
            {
                in_N_idx = i;
            }
        }
        let mut in_S_idx = invalid;
        for i in 0..self.in_direction.len()
        {
            if self.in_direction[i] == "S"
            {
                in_S_idx = i;
            }
        }
        let mut in_E_idx = invalid;
        for i in 0..self.in_direction.len()
        {
            if self.in_direction[i] == "E"
            {
                in_E_idx = i;
            }
        }
        let mut in_W_idx = invalid;
        for i in 0..self.in_direction.len()
        {
            if self.in_direction[i] == "W"
            {
                in_W_idx = i;
            }
        }
        let mut in_L_idx = invalid;
        for i in 0..self.in_direction.len()
        {
            if self.in_direction[i] == "L"
            {
                in_L_idx = i;
            }
        }




        let mut out_N_idx = invalid;
        for i in 0..self.out_direction.len()
        {
            if self.out_direction[i] == "N"
            {
                out_N_idx = i;
            }
        }
        let mut out_S_idx = invalid;
        for i in 0..self.out_direction.len()
        {
            if self.out_direction[i] == "S"
            {
                out_S_idx = i;
            }
        }
        let mut out_E_idx = invalid;
        for i in 0..self.out_direction.len()
        {
            if self.out_direction[i] == "E"
            {
                out_E_idx = i;
            }
        }
        let mut out_W_idx = invalid;
        for i in 0..self.out_direction.len()
        {
            if self.out_direction[i] == "W"
            {
                out_W_idx = i;
            }
        }
        let mut out_L_idx = invalid;
        for i in 0..self.out_direction.len()
        {
            if self.out_direction[i] == "L"
            {
                out_L_idx = i;
            }
        }


        loop
        {
            for j in 0..self.in_len
            {
                let next_data = self.in_stream[j].dequeue(&self.time).unwrap().data;
                let dst_x = next_data / self.y_dim;
                let dst_y = next_data % self.y_dim;

                buffer.push(next_data);

                if buffer.len() == 0
                {

                }


                if dst_x == self.x && dst_y == self.y // exit local port
                {
                    if out_L_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_L_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x == self.x && dst_y < self.y // exit W port
                {
                    if out_W_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_W_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x < self.x && dst_y < self.y // exit N port
                {
                    if out_N_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_N_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x < self.x && dst_y == self.y // exit N port
                {
                    if out_N_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_N_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x < self.x && dst_y > self.y // exit N port
                {
                    if out_N_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_N_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }


                } else if dst_x == self.x && dst_y > self.y // exit E port
                {
                    if out_E_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_E_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x > self.x && dst_y > self.y // exit S port
                {
                    if out_S_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_S_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x > self.x && dst_y == self.y // exit S port
                {
                    if out_S_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_S_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
                        self.time.incr_cycles(1);
                    }

                } else if dst_x > self.x && dst_y < self.y // exit S port
                {
                    if out_S_idx == invalid
                    {
                        panic!("Wrong!");
                    } else {
                        let curr_time = self.time.tick();
                        self.out_stream[out_S_idx].enqueue(&self.time, ChannelElement::new(curr_time+1 as u64, next_data.clone())).unwrap();
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
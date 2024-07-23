// use dam::channel::PeekResult;
// use dam::context_tools::*;
// use dam::{
//     channel::{Receiver, Sender},
//     context::Context,
//     templates::{ops::ALUOp, pcu::*},
//     types::DAMType,
// };

// #[context_macro]
// pub struct to_router_adapter<A: Clone> {
//     pub in_stream: Vec<Receiver<usize>>,
//     pub in_len: usize,
//     pub out_stream: Vec<Sender<usize>>,
//     pub num_input: usize,
//     pub num_vc: usize,
//     pub dummy: A,
// }

// impl<A: DAMType> to_router_adapter<A>
// where
// to_router_adapter<A>: Context,
// {
//     pub fn new(
//         in_stream: Vec<Receiver<usize>>,
//         in_len: usize,
//         out_stream: Vec<Sender<usize>>,
//         num_input: usize,
//         num_vc: usize,
//         dummy: A,
//     ) -> Self {
//         let to_router_adapter = to_router_adapter {
//             in_stream,
//             in_len,
//             out_stream,
//             num_input,
//             num_vc,
//             dummy,
//             context_info: Default::default(),
//         };
//         for i in 0..in_len
//         {
//             let idx: usize = i.try_into().unwrap();
//             to_router_adapter.in_stream[idx].attach_receiver(&to_router_adapter);
//         }
//         for i in 0..num_vc
//         {
//             to_router_adapter.out_stream[i].attach_sender(&to_router_adapter);
//         }

//         to_router_adapter
//     }
// }

// impl<A: DAMType + num::Num> Context for to_router_adapter<A> {
//     fn run(&mut self) {
//         for _ in 0..self.num_input
//         {
//             for j in 0..self.in_len
//             {
//                 let idx: usize = j.try_into().unwrap();
//                 let in_data = self.in_stream[idx].dequeue(&self.time).unwrap().data;

//                 let curr_time = self.time.tick();
//                 self.out_stream[0].enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
//             }
//         }
//         self.time.incr_cycles(1);
//     }
// }








// #[context_macro]
// pub struct from_router_adapter<A: Clone> {
//     pub in_stream: Vec<Receiver<usize>>,
//     pub out_stream: Vec<Sender<usize>>,
//     pub out_len: usize,
//     pub x: usize,
//     pub y: usize,
//     pub num_input: usize,
//     pub num_vc: usize,
//     pub dummy: A,
// }

// impl<A: DAMType> from_router_adapter<A>
// where
// from_router_adapter<A>: Context,
// {
//     pub fn new(
//         in_stream: Vec<Receiver<usize>>,
//         out_stream: Vec<Sender<usize>>,
//         out_len: usize,
//         x: usize,
//         y: usize,
//         num_input: usize,
//         num_vc: usize,
//         dummy: A,
//     ) -> Self {
//         let from_router_adapter = from_router_adapter {
//             in_stream,
//             out_stream,
//             out_len,
//             x,
//             y,
//             num_input,
//             num_vc,
//             dummy,
//             context_info: Default::default(),
//         };
//         for i in 0..num_vc
//         {
//             from_router_adapter.in_stream[i].attach_receiver(&from_router_adapter);
//         }
        
//         for i in 0..out_len
//         {
//             let idx: usize = i.try_into().unwrap();
//             from_router_adapter.out_stream[idx].attach_sender(&from_router_adapter);
//         }

//         from_router_adapter
//     }
// }

// impl<A: DAMType + num::Num> Context for from_router_adapter<A> {
//     fn run(&mut self)
//     {   
//         for i in 0..self.num_input
//         {
//             for j in 0..self.out_len
//             {
//                 let in_data = self.in_stream[0].dequeue(&self.time).unwrap().data;

//                 let curr_time = self.time.tick();
//                 let idx: usize = j.try_into().unwrap();
//                 self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time + 1, in_data.clone())).unwrap();
//             }
//         }
//         self.time.incr_cycles(1);

//         // let mut received_cnt = 0;
//         // let mut iter_out_len = 0;
//         // let mut in_closed = vec![];
//         // let invalid = 999999;

//         // for _ in 0..self.num_vc
//         // {
//         //     in_closed.push(false);
//         // }

//         // let mut aaa = 0;

//         // loop
//         // {
//         //     let mut data_vec = vec![];

//         //     for i in 0..self.num_vc
//         //     {
//         //         if !in_closed[i]
//         //         {
//         //             let peek_result = self.in_stream[i].peek();
//         //             match peek_result {
//         //                 PeekResult::Something(_) =>
//         //                 {
//         //                     let data = self.in_stream[i].dequeue(&self.time).unwrap().data;
//         //                     data_vec.push(data);
//         //                 },
//         //                 PeekResult::Nothing(_) => 
//         //                 {
//         //                     data_vec.push(invalid);
//         //                 },
//         //                 PeekResult::Closed =>
//         //                 {
//         //                     data_vec.push(invalid);
//         //                     in_closed[i] = true;
//         //                 },
//         //             }
//         //         }
//         //     }


//         //     if aaa <= 200 && self.x == 0 && self.y == 0
//         //     {
//         //         println!("x{}, y{}, data_vec{:?}, in_closed{:?}", self.x, self.y, data_vec, in_closed);
//         //         aaa += 1;
//         //     }

//         //     for i in 0..data_vec.len()
//         //     {
//         //         if data_vec[i] != invalid
//         //         {
//         //             let curr_time = self.time.tick();
//         //             self.out_stream[iter_out_len].enqueue(&self.time, ChannelElement::new(curr_time + 1, data_vec[i].clone())).unwrap();
//         //             iter_out_len = (iter_out_len + 1) % self.out_len;
//         //             received_cnt += 1;
    
//         //             if received_cnt >= self.num_input * self.out_len
//         //             {
//         //                 self.time.incr_cycles(1);
//         //                 return;
//         //             }
//         //         }  
//         //     }
//         // }
        
//     }
// }














use dam::context_tools::*;
use dam::{
    channel::{Receiver, Sender},
    context::Context,
    templates::{ops::ALUOp, pcu::*},
    types::DAMType,
};

#[context_macro]
pub struct to_router_adapter<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream: Sender<usize>,
    pub num_input: usize,
    pub dummy: A,
}

impl<A: DAMType> to_router_adapter<A>
where
to_router_adapter<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream: Sender<usize>,
        num_input: usize,
        dummy: A,
    ) -> Self {
        let to_router_adapter = to_router_adapter {
            in_stream,
            in_len,
            out_stream,
            num_input,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {
            let idx: usize = i.try_into().unwrap();
            to_router_adapter.in_stream[idx].attach_receiver(&to_router_adapter);
        }
        to_router_adapter.out_stream.attach_sender(&to_router_adapter);

        to_router_adapter
    }
}

impl<A: DAMType + num::Num> Context for to_router_adapter<A> {
    fn run(&mut self) {
        for i in 0..self.num_input
        {
            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                let in_data = self.in_stream[idx].dequeue(&self.time).unwrap().data;

                let curr_time = self.time.tick();
                self.out_stream.enqueue(&self.time, ChannelElement::new(curr_time, in_data.clone())).unwrap();
            }
            self.time.incr_cycles(1);
        }
    }
}








#[context_macro]
pub struct from_router_adapter<A: Clone> {
    pub in_stream: Receiver<usize>,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub num_input: usize,
    pub dummy: A,
}

impl<A: DAMType> from_router_adapter<A>
where
from_router_adapter<A>: Context,
{
    pub fn new(
        in_stream: Receiver<usize>,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        num_input: usize,
        dummy: A,
    ) -> Self {
        let from_router_adapter = from_router_adapter {
            in_stream,
            out_stream,
            out_len,
            num_input,
            dummy,
            context_info: Default::default(),
        };
        from_router_adapter.in_stream.attach_receiver(&from_router_adapter);
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            from_router_adapter.out_stream[idx].attach_sender(&from_router_adapter);
        }

        from_router_adapter
    }
}

impl<A: DAMType + num::Num> Context for from_router_adapter<A> {
    fn run(&mut self) {
        for i in 0..self.num_input
        {
            for j in 0..self.out_len
            {
                let in_data = self.in_stream.dequeue(&self.time).unwrap().data;

                let curr_time = self.time.tick();
                let idx: usize = j.try_into().unwrap();
                self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time, in_data.clone())).unwrap();
            }
            self.time.incr_cycles(1);
        }
    }
}



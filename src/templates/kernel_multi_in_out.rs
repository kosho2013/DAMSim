use dam::context_tools::*;

#[context_macro]
pub struct kernel_multi_in_out<A: Clone> {
    pub in_stream: Vec<Receiver<usize>>,
    pub in_len: usize,
    pub out_stream: Vec<Sender<usize>>,
    pub out_len: usize,
    pub latency: usize,
    pub init_inverval: usize,
    pub loop_bound: usize,
    pub dummy: A
}

impl<A: DAMType> kernel_multi_in_out<A>
where
kernel_multi_in_out<A>: Context,
{
    pub fn new(
        in_stream: Vec<Receiver<usize>>,
        in_len: usize,
        out_stream: Vec<Sender<usize>>,
        out_len: usize,
        latency: usize,
        init_inverval: usize,
        loop_bound: usize,
        dummy: A,
    ) -> Self {
        let kernel_multi_in_out = kernel_multi_in_out {
            in_stream,
            in_len,
            out_stream,
            out_len,
            latency,
            init_inverval,
            loop_bound,
            dummy,
            context_info: Default::default(),
        };
        for i in 0..in_len
        {   
            let idx: usize = i.try_into().unwrap();
            kernel_multi_in_out.in_stream[idx].attach_receiver(&kernel_multi_in_out);
        }
        for i in 0..out_len
        {
            let idx: usize = i.try_into().unwrap();
            kernel_multi_in_out.out_stream[idx].attach_sender(&kernel_multi_in_out);
        }

        kernel_multi_in_out
    }
}

impl<A: DAMType + num::Num> Context for kernel_multi_in_out<A> {
    fn run(&mut self) {
		
        for i in 0..self.loop_bound
        {

            let mut in_vec = vec![];

            for j in 0..self.in_len
            {
                let idx: usize = j.try_into().unwrap();
                in_vec.push(self.in_stream[idx].dequeue(&self.time));
            }

            let in_data = in_vec.remove(0).unwrap().data;
            
            let curr_time = self.time.tick();
            for j in 0..self.out_len
            {
                let idx: usize = j.try_into().unwrap();
                self.out_stream[idx].enqueue(&self.time, ChannelElement::new(curr_time + self.latency as u64, in_data.clone())).unwrap();
            }
            
            self.time.incr_cycles(self.init_inverval as u64);
        
        }
    }
}

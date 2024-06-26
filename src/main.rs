extern crate protobuf;

mod proto_driver;
pub mod templates;
pub mod utils;

use std::collections::HashMap;
use std::{env, process};
use std::{fs, time::Instant};
use std::fs::{read_to_string, File};
use std::io::{self, BufRead, BufReader};
use dam::channel::{ChannelElement, Receiver};
use dam::templates::datastore::Behavior;
use dam::templates::pmu::{PMUReadBundle, PMUWriteBundle, PMU};
use dam::utility_contexts::{ConsumerContext, FunctionContext, GeneratorContext, PrinterContext};
use proto_driver::proto_headers::setup::System;
use prost::Message;
use dam::{logging::LogEvent, simulation::*};
use templates::kernel::kernel;
use templates::kernel_multi_in_out::kernel_multi_in_out;
use templates::my_pcu::{make_simd_pcu, make_systolic_pcu};
use templates::pcu_adapter::{simd_pcu_adapter_downstream, simd_pcu_adapter_upstream, systolic_pcu_adapter_downstream, systolic_pcu_adapter_upstream};
use templates::pmu_adapter::{pmu_adapter_downstream, pmu_adapter_upstream};


fn main() {
	let args: Vec<String> = env::args().collect();
	println!("{:?}", args);

    let system_path = format!("{}{}", &args[1], "/system.bin");
	let experiment_num_input = format!("{}", &args[2]);

	let parts: Vec<&str> = experiment_num_input.split('_').collect();
	let experiment = parts[0].parse::<usize>().unwrap();
	let num_input = parts[1].parse::<usize>().unwrap();

	// get system
	let system = {
        let file_contents = fs::read(system_path).unwrap();
        System::decode(file_contents.as_slice()).unwrap()
    };
	println!("{:?}", system);

	let accelerator = system.accelerator.unwrap();
	let lane_dim: usize = accelerator.lane_dim as usize;
	let stage_dim: usize = accelerator.stage_dim as usize;
	let sram_l1_cap: usize = accelerator.sram_l1_cap as usize;
	let word: usize = accelerator.word as usize;

	let num_vec_per_pmu = sram_l1_cap / lane_dim / word;
	println!("num_vec_per_pmu {}", num_vec_per_pmu);


	println!("lane_dim {}", lane_dim);
	println!("stage_dim {}", stage_dim);

	let mut Memory_Latency = vec![];
	let mut Network_Latency = vec![];
	let mut num_tile: usize = 0;
	
	let mapping_path = format!("{}{}", &args[1], "/mapping.txt");
	let lines = read_to_string(mapping_path).unwrap();


	let mut dfmodel_time: Vec<usize> = vec![];
	let mut experiment_time: Vec<usize> = Vec::new();


	

	for line in lines.lines() {
		if line.starts_with("num_tile") 
		{
			let tmp = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2: f32 = tmp;
			let tmp3: usize = tmp2.round() as usize;
			num_tile = tmp3;
		}
	}

	

 

	for line in lines.lines() {
		if line.starts_with("Memory_Latency[") 
		{ 
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2 = tmp / num_tile as f32;
			let tmp3: usize = tmp2.round() as usize;
			Memory_Latency.push(tmp3);
		}

		if line.starts_with("Network_Latency[") 
		{
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2 = tmp / num_tile as f32;
			let tmp3: usize = tmp2.round() as usize;
			Network_Latency.push(tmp3);
		}

		if line.starts_with("Per_Config_II[") 
		{ 
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2: f32 = tmp / num_tile as f32;
			let tmp3: usize = tmp2.round() as usize;
			dfmodel_time.push(tmp3);
		}
	}


	let num_config = Network_Latency.len();





	

	println!("Memory_Latency {:?}", Memory_Latency);
	println!("Network_Latency {:?}", Network_Latency);
	println!("num_tile {}", num_tile);
	println!("num_input {}", num_input);
	println!("num_config {}", num_config);


	



	for i in 0..num_config
	{
		println!("config: {}", i);



		let mut pcu_x: Vec<usize> = vec![];
		let mut pcu_y: Vec<usize> = vec![];
		let mut pcu_cycle: Vec<usize> = vec![];
		let mut pcu_sender_vec: Vec<Vec<usize>> = vec![];
		let mut pcu_receiver_vec: Vec<Vec<usize>> = vec![];

		let mut pcu_SIMD_or_Systolic: Vec<&str> = vec![];
		let mut pcu_M: Vec<usize> = vec![];
		let mut pcu_K: Vec<usize> = vec![];
		let mut pcu_N: Vec<usize> = vec![];
		

		let mut pmu_x: Vec<usize> = vec![];
		let mut pmu_y: Vec<usize> = vec![];
		let mut pmu_cycle: Vec<usize> = vec![];
		let mut pmu_sender_vec: Vec<Vec<usize>>  = vec![];
		let mut pmu_receiver_vec: Vec<Vec<usize>>  = vec![];

		let mut num_of_connections = 0;

		for line in lines.lines() {
			let str = format!("num_of_connections config {}", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[tmp.len() - 1].parse().unwrap();
				num_of_connections = tmp1;
			}	
		}

		for line in lines.lines() {
			let str = format!("pcu config {}", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();

				pcu_x.push(tmp[4].parse().unwrap());
				pcu_y.push(tmp[5].parse().unwrap());
				pcu_cycle.push(tmp[6].parse().unwrap());
				pcu_SIMD_or_Systolic.push(tmp[7]);
				pcu_M.push(tmp[8].parse().unwrap());
				pcu_K.push(tmp[9].parse().unwrap());
				pcu_N.push(tmp[10].parse().unwrap());

				let str = "receiver";
				let mut j = 12;
				
				let mut tmp_sender_vec: Vec<usize> = vec![];
				while tmp[j] != str 
				{
					tmp_sender_vec.push(tmp[j].parse().unwrap());
					j += 1;
				}
				pcu_sender_vec.push(tmp_sender_vec);
				
				let mut tmp_receiver_vec: Vec<usize> = vec![];
				for k in j+1..tmp.len()
				{
					tmp_receiver_vec.push(tmp[k].parse().unwrap());
				}
				pcu_receiver_vec.push(tmp_receiver_vec);

			}	
		}	
		



		for line in lines.lines() {
			let str = format!("pmu config {}", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();

				pmu_x.push(tmp[4].parse().unwrap());
				pmu_y.push(tmp[5].parse().unwrap());
				pmu_cycle.push(tmp[6].parse().unwrap());

				let str = "receiver";
				let mut j = 8;
				
				let mut tmp_sender_vec: Vec<usize> = vec![];
				while tmp[j] != str 
				{
					tmp_sender_vec.push(tmp[j].parse().unwrap());
					j += 1;
				}
				pmu_sender_vec.push(tmp_sender_vec);
				
				let mut tmp_receiver_vec: Vec<usize> = vec![];
				for k in j+1..tmp.len()
				{
					tmp_receiver_vec.push(tmp[k].parse().unwrap());
				}
				pmu_receiver_vec.push(tmp_receiver_vec);

			}	
		}


		let mut kernel_latency: Vec<usize> = vec![];
		for line in lines.lines() {
			let str = format!("kernel latency config {}", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[tmp.len() - 1].parse().unwrap();
				kernel_latency.push(tmp1);
			}
		}


		




		println!("kernel_latency{:?}", kernel_latency);
		println!("pcu_x{:?}", pcu_x);
		println!("pcu_y{:?}", pcu_y);
		println!("pcu_cycle {:?}", pcu_cycle);
		println!("pcu_sender_vec {:?}", pcu_sender_vec);
		println!("pcu_receiver_vec {:?}", pcu_receiver_vec);

		println!("pcu_SIMD_or_Systolic {:?}", pcu_SIMD_or_Systolic);
		println!("pcu_M {:?}", pcu_M);
		println!("pcu_K {:?}", pcu_K);
		println!("pcu_N {:?}", pcu_N);

		println!("pmu_x{:?}", pmu_x);
		println!("pmu_y{:?}", pmu_y);
		println!("pmu_cycle {:?}", pmu_cycle);
		println!("pmu_sender_vec {:?}", pmu_sender_vec);
		println!("pmu_receiver_vec {:?}", pmu_receiver_vec);

		println!("num_of_connections {}", num_of_connections);

		println!("\n\n\n\n");





		












		// experiment 1
		let num_kernel: usize = kernel_latency.len().try_into().unwrap();

		let mut parent = ProgramBuilder::default();

		// channels
		let mut sender_map_net: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_net: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..2
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_net.insert(j, sender);
			receiver_map_net.insert(j, receiver);
		}

		let mut sender_map_mem: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_mem: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..2
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_mem.insert(j, sender);
			receiver_map_mem.insert(j, receiver);
		}

		let mut sender_map_comp: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_comp: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..num_kernel+1
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_comp.insert(j.try_into().unwrap(), sender);
			receiver_map_comp.insert(j.try_into().unwrap(), receiver);
		}



		// off-chip memory
		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_mem.remove(&0).unwrap());
		parent.add_child(input);

		let memory = kernel::new(receiver_map_mem.remove(&0).unwrap(), sender_map_mem.remove(&1).unwrap(), Memory_Latency[i] as usize, Memory_Latency[i] as usize, num_input as usize);
		parent.add_child(memory);

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_mem.remove(&1).unwrap());
		parent.add_child(output);



		// off-chip network
		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_net.remove(&0).unwrap());
		parent.add_child(input);

		let network = kernel::new(receiver_map_net.remove(&0).unwrap(), sender_map_net.remove(&1).unwrap(), Network_Latency[i] as usize, Network_Latency[i] as usize, num_input as usize);
		parent.add_child(network);

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_net.remove(&1).unwrap());
		parent.add_child(output);


		// on-chip compute
		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_comp.remove(&0).unwrap());
		parent.add_child(input);

		for j in 0..num_kernel
		{
			let compute: kernel<usize> = kernel::new(receiver_map_comp.remove(&j).unwrap(), sender_map_comp.remove(&(j+1)).unwrap(), kernel_latency[j as usize] as usize, kernel_latency[j as usize] as usize, num_input as usize);
			parent.add_child(compute);
		}

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_comp.remove(&num_kernel).unwrap());
		parent.add_child(output);



		if experiment == 1
		{
			println!("experiment 1 ***************************************************************************************");

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


			let time = executed.elapsed_cycles().unwrap();
			let time_tmp: f32 = time as f32 / num_input as f32;
			experiment_time.push(time_tmp as usize);
		}
		

















		// experiment 2		
		let mut parent = ProgramBuilder::default();

		// off-chip memory
		let mut sender_map_mem: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_mem: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..2
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_mem.insert(j, sender);
			receiver_map_mem.insert(j, receiver);
		}


		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_mem.remove(&0).unwrap());
		parent.add_child(input);

		let memory = kernel::new(receiver_map_mem.remove(&0).unwrap(), sender_map_mem.remove(&1).unwrap(), Memory_Latency[i] as usize, Memory_Latency[i] as usize, num_input as usize);
		parent.add_child(memory);

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_mem.remove(&1).unwrap());
		parent.add_child(output);



		// off-chip network
		let mut sender_map_net: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_net: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..2
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_net.insert(j, sender);
			receiver_map_net.insert(j, receiver);
		}

		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_net.remove(&0).unwrap());
		parent.add_child(input);

		let network = kernel::new(receiver_map_net.remove(&0).unwrap(), sender_map_net.remove(&1).unwrap(), Network_Latency[i] as usize, Network_Latency[i] as usize, num_input as usize);
		parent.add_child(network);

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_net.remove(&1).unwrap());
		parent.add_child(output);





		let mut sender_map: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..num_of_connections
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map.insert(j, sender);
			receiver_map.insert(j, receiver);
		}





		// on-chip pcu contexts
		for j in 0..pcu_x.len()
		{
			let mut pcu_x_tmp = pcu_x[j];
			let mut pcu_y_tmp = pcu_y[j];
			let mut pcu_cycle_tmp = pcu_cycle[j];
			let mut pcu_sender_vec_tmp = vec![];
			for n in 0..pcu_sender_vec[j].len()
			{
				pcu_sender_vec_tmp.push(pcu_sender_vec[j][n]);
			}		
			let mut pcu_receiver_vec_tmp = vec![];
			for n in 0..pcu_receiver_vec[j].len()
			{
				pcu_receiver_vec_tmp.push(pcu_receiver_vec[j][n]);
			}


			let simd_or_systolic = pcu_SIMD_or_Systolic[j];
			let M = pcu_M[j];
			let K = pcu_K[j];
			let N = pcu_N[j];



			let no_connection = 999999;
			if pcu_receiver_vec_tmp[0] == no_connection
			{
				let mut tmp_sender_vec = vec![];
				for k in 0..pcu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pcu_sender_vec_tmp[k]).unwrap());
				}
				let (sender, receiver) = parent.bounded(1024);
				let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
				let gen = GeneratorContext::new(iter, sender);
				parent.add_child(gen);


				let mut tmp_receiver_vec = vec![];
				tmp_receiver_vec.push(receiver);
				


				if simd_or_systolic == "SIMD"
				{					
					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					
					let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_upstream);

					let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
					parent.add_child(pcu);

					let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_downstream);

				} else if simd_or_systolic == "Systolic"
				{
					// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, pcu_cycle_tmp as usize, pcu_cycle_tmp as usize, num_input as usize);
					// parent.add_child(kernel);


					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					let (sender3, receiver3) = parent.bounded(1024);
					let (sender4, receiver4) = parent.bounded(1024);

					let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_upstream);

					let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
					parent.add_child(pcu_lane);

					let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
					parent.add_child(pcu_stage);

					let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_downstream);




				} else {
					panic!("Wrong!");
				}




			} else if pcu_sender_vec_tmp[0] == no_connection
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pcu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pcu_receiver_vec_tmp[k]).unwrap());
				}

				let (sender, receiver) = parent.bounded(1024);
				let mut tmp_sender_vec: Vec<dam::channel::Sender<usize>> = vec![];
				tmp_sender_vec.push(sender);
				
				


				if simd_or_systolic == "SIMD"
				{					
					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					
					let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_upstream);

					let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
					parent.add_child(pcu);

					let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_downstream);

				} else if simd_or_systolic == "Systolic"
				{
					// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, pcu_cycle_tmp as usize, pcu_cycle_tmp as usize, num_input as usize);
					// parent.add_child(kernel);


					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					let (sender3, receiver3) = parent.bounded(1024);
					let (sender4, receiver4) = parent.bounded(1024);

					let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_upstream);

					let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
					parent.add_child(pcu_lane);

					let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
					parent.add_child(pcu_stage);

					let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_downstream);




				} else {
					panic!("Wrong!");
				}





				let con = ConsumerContext::new(receiver);
				parent.add_child(con);


			} else if pcu_receiver_vec_tmp[0] == no_connection && pcu_sender_vec_tmp[0] == no_connection
			{
				panic!("Wrong!");


			} else
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pcu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pcu_receiver_vec_tmp[k]).unwrap());
				}

				let mut tmp_sender_vec = vec![];
				for k in 0..pcu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pcu_sender_vec_tmp[k]).unwrap());
				}



				if simd_or_systolic == "SIMD"
				{					
					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					
					let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_upstream);

					let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
					parent.add_child(pcu);

					let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_downstream);

				} else if simd_or_systolic == "Systolic"
				{
					// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, pcu_cycle_tmp as usize, pcu_cycle_tmp as usize, num_input as usize);
					// parent.add_child(kernel);




					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					let (sender3, receiver3) = parent.bounded(1024);
					let (sender4, receiver4) = parent.bounded(1024);

					let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_upstream);

					let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
					parent.add_child(pcu_lane);

					let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
					parent.add_child(pcu_stage);

					let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_downstream);









				} else {
					panic!("Wrong!");
				}


			}
		}







		// on-chip pmu contexts
		for j in 0..pmu_x.len()
		{
			let mut pmu_x_tmp = pmu_x[j];
			let mut pmu_y_tmp = pmu_y[j];
			let mut pmu_cycle_tmp = pmu_cycle[j];
			let mut pmu_sender_vec_tmp = vec![];
			for n in 0..pmu_sender_vec[j].len()
			{
				pmu_sender_vec_tmp.push(pmu_sender_vec[j][n]);
			}		
			let mut pmu_receiver_vec_tmp = vec![];
			for n in 0..pmu_receiver_vec[j].len()
			{
				pmu_receiver_vec_tmp.push(pmu_receiver_vec[j][n]);
			}
			
			let no_connection = 999999;
			if pmu_receiver_vec_tmp[0] == no_connection
			{
				let mut tmp_sender_vec = vec![];
				for k in 0..pmu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pmu_sender_vec_tmp[k]).unwrap());
				}
				let (sender, receiver) = parent.bounded(1024);
				let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
				let gen = GeneratorContext::new(iter, sender);
				parent.add_child(gen);


				let mut tmp_receiver_vec = vec![];
				tmp_receiver_vec.push(receiver);
				
				let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, 1, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, pmu_cycle_tmp as usize, pmu_cycle_tmp as usize, num_input as usize);
				parent.add_child(kernel);

			} else if pmu_sender_vec_tmp[0] == no_connection
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pmu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pmu_receiver_vec_tmp[k]).unwrap());
				}

				let (sender, receiver) = parent.bounded(1024);
				let mut tmp_sender_vec: Vec<dam::channel::Sender<usize>> = vec![];
				tmp_sender_vec.push(sender);
				
				let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, tmp_sender_vec, 1, pmu_cycle_tmp as usize, pmu_cycle_tmp as usize, num_input as usize);
				parent.add_child(kernel);

				let con = ConsumerContext::new(receiver);
				parent.add_child(con);


			} else if pmu_receiver_vec_tmp[0] == no_connection && pmu_sender_vec_tmp[0] == no_connection
			{
				panic!("Wrong!");


			} else
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pmu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pmu_receiver_vec_tmp[k]).unwrap());
				}

				let mut tmp_sender_vec = vec![];
				for k in 0..pmu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pmu_sender_vec_tmp[k]).unwrap());
				}
				let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, pmu_cycle_tmp as usize, pmu_cycle_tmp as usize, num_input as usize);
				parent.add_child(kernel);



			}
		}














		if experiment == 2
		{
			println!("experiment 2 ***************************************************************************************");

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


			let time = executed.elapsed_cycles().unwrap();
			let time_tmp: f32 = time as f32 / num_input as f32;
			experiment_time.push(time_tmp as usize);
		}
















		








		// experiment 3
		let mut parent = ProgramBuilder::default();

		// off-chip memory
		let mut sender_map_mem: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_mem: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..2
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_mem.insert(j, sender);
			receiver_map_mem.insert(j, receiver);
		}


		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_mem.remove(&0).unwrap());
		parent.add_child(input);

		let memory = kernel::new(receiver_map_mem.remove(&0).unwrap(), sender_map_mem.remove(&1).unwrap(), Memory_Latency[i] as usize, Memory_Latency[i] as usize, num_input as usize);
		parent.add_child(memory);

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_mem.remove(&1).unwrap());
		parent.add_child(output);



		// off-chip network
		let mut sender_map_net: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map_net: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..2
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map_net.insert(j, sender);
			receiver_map_net.insert(j, receiver);
		}

		let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
		let input = GeneratorContext::new(iter, sender_map_net.remove(&0).unwrap());
		parent.add_child(input);

		let network = kernel::new(receiver_map_net.remove(&0).unwrap(), sender_map_net.remove(&1).unwrap(), Network_Latency[i] as usize, Network_Latency[i] as usize, num_input as usize);
		parent.add_child(network);

		let output: ConsumerContext<_> = ConsumerContext::new(receiver_map_net.remove(&1).unwrap());
		parent.add_child(output);





		let mut sender_map: HashMap<usize, dam::channel::Sender<_>> = HashMap::new();
		let mut receiver_map: HashMap<usize, dam::channel::Receiver<_>> = HashMap::new();
		for j in 0..num_of_connections
		{
			let (sender, receiver) = parent.bounded(1024);
			sender_map.insert(j, sender);
			receiver_map.insert(j, receiver);
		}





		// on-chip pcu contexts
		for j in 0..pcu_x.len()
		{
			let mut pcu_x_tmp = pcu_x[j];
			let mut pcu_y_tmp = pcu_y[j];
			let mut pcu_cycle_tmp = pcu_cycle[j];
			let mut pcu_sender_vec_tmp = vec![];
			for n in 0..pcu_sender_vec[j].len()
			{
				pcu_sender_vec_tmp.push(pcu_sender_vec[j][n]);
			}		
			let mut pcu_receiver_vec_tmp = vec![];
			for n in 0..pcu_receiver_vec[j].len()
			{
				pcu_receiver_vec_tmp.push(pcu_receiver_vec[j][n]);
			}


			let simd_or_systolic = pcu_SIMD_or_Systolic[j];
			let M = pcu_M[j];
			let K = pcu_K[j];
			let N = pcu_N[j];



			let no_connection = 999999;
			if pcu_receiver_vec_tmp[0] == no_connection
			{
				let mut tmp_sender_vec = vec![];
				for k in 0..pcu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pcu_sender_vec_tmp[k]).unwrap());
				}
				let (sender, receiver) = parent.bounded(1024);
				let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
				let gen = GeneratorContext::new(iter, sender);
				parent.add_child(gen);


				let mut tmp_receiver_vec = vec![];
				tmp_receiver_vec.push(receiver);
				


				if simd_or_systolic == "SIMD"
				{					
					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					
					let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_upstream);

					let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
					parent.add_child(pcu);

					let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_downstream);

				} else if simd_or_systolic == "Systolic"
				{
					// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, pcu_cycle_tmp as usize, pcu_cycle_tmp as usize, num_input as usize);
					// parent.add_child(kernel);


					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					let (sender3, receiver3) = parent.bounded(1024);
					let (sender4, receiver4) = parent.bounded(1024);

					let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_upstream);

					let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
					parent.add_child(pcu_lane);

					let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
					parent.add_child(pcu_stage);

					let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_downstream);




				} else {
					panic!("Wrong!");
				}




			} else if pcu_sender_vec_tmp[0] == no_connection
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pcu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pcu_receiver_vec_tmp[k]).unwrap());
				}

				let (sender, receiver) = parent.bounded(1024);
				let mut tmp_sender_vec: Vec<dam::channel::Sender<usize>> = vec![];
				tmp_sender_vec.push(sender);
				
				


				if simd_or_systolic == "SIMD"
				{					
					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					
					let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_upstream);

					let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
					parent.add_child(pcu);

					let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_downstream);

				} else if simd_or_systolic == "Systolic"
				{
					// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, pcu_cycle_tmp as usize, pcu_cycle_tmp as usize, num_input as usize);
					// parent.add_child(kernel);


					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					let (sender3, receiver3) = parent.bounded(1024);
					let (sender4, receiver4) = parent.bounded(1024);

					let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_upstream);

					let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
					parent.add_child(pcu_lane);

					let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
					parent.add_child(pcu_stage);

					let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_downstream);




				} else {
					panic!("Wrong!");
				}





				let con = ConsumerContext::new(receiver);
				parent.add_child(con);


			} else if pcu_receiver_vec_tmp[0] == no_connection && pcu_sender_vec_tmp[0] == no_connection
			{
				panic!("Wrong!");


			} else
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pcu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pcu_receiver_vec_tmp[k]).unwrap());
				}

				let mut tmp_sender_vec = vec![];
				for k in 0..pcu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pcu_sender_vec_tmp[k]).unwrap());
				}



				if simd_or_systolic == "SIMD"
				{					
					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					
					let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_upstream);

					let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
					parent.add_child(pcu);

					let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize);
					parent.add_child(simd_pcu_adapter_downstream);

				} else if simd_or_systolic == "Systolic"
				{
					// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, pcu_cycle_tmp as usize, pcu_cycle_tmp as usize, num_input as usize);
					// parent.add_child(kernel);




					let (sender1, receiver1) = parent.bounded(1024);
					let (sender2, receiver2) = parent.bounded(1024);
					let (sender3, receiver3) = parent.bounded(1024);
					let (sender4, receiver4) = parent.bounded(1024);

					let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_upstream);

					let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
					parent.add_child(pcu_lane);

					let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
					parent.add_child(pcu_stage);

					let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim);
					parent.add_child(systolic_pcu_adapter_downstream);









				} else {
					panic!("Wrong!");
				}


			}
		}







		// on-chip pmu contexts
		for j in 0..pmu_x.len()
		{
			let mut pmu_x_tmp = pmu_x[j];
			let mut pmu_y_tmp = pmu_y[j];
			let mut pmu_cycle_tmp = pmu_cycle[j];
			let mut pmu_sender_vec_tmp = vec![];
			for n in 0..pmu_sender_vec[j].len()
			{
				pmu_sender_vec_tmp.push(pmu_sender_vec[j][n]);
			}		
			let mut pmu_receiver_vec_tmp = vec![];
			for n in 0..pmu_receiver_vec[j].len()
			{
				pmu_receiver_vec_tmp.push(pmu_receiver_vec[j][n]);
			}
			
			let no_connection = 999999;
			if pmu_receiver_vec_tmp[0] == no_connection
			{
				let mut tmp_sender_vec = vec![];
				for k in 0..pmu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pmu_sender_vec_tmp[k]).unwrap());
				}
				let (sender, receiver) = parent.bounded(1024);
				let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
				let gen = GeneratorContext::new(iter, sender);
				parent.add_child(gen);


				let mut tmp_receiver_vec = vec![];
				tmp_receiver_vec.push(receiver);
				
				// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, 1, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, pmu_cycle_tmp as usize, pmu_cycle_tmp as usize, num_input as usize);
				// parent.add_child(kernel);





				let (wr_addr_sender, wr_addr_receiver) = parent.bounded(1024);
				let (wr_data_sender, wr_data_receiver) = parent.bounded(1024);
				let (ack_sender, ack_receiver) = parent.bounded(1024);
				let (rd_addr_sender, rd_addr_receiver) = parent.bounded(1024);
				let (rd_data_sender, rd_data_receiver) = parent.bounded(1024);

				let pmu_adapter_upstream = pmu_adapter_upstream::new(tmp_receiver_vec, 1, wr_addr_sender, wr_data_sender, num_input as usize, pmu_cycle_tmp);
				parent.add_child(pmu_adapter_upstream);

				let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
					num_vec_per_pmu,
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
				let tmp = pmu_cycle_tmp * num_input;
				rd_addr_gen.set_run(move |time| {
					for idx in 0..tmp
					{
						ack_receiver.dequeue(time).unwrap();
						let curr_time = time.tick();
						rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
					}
				});
				parent.add_child(rd_addr_gen);




				
				let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, num_input as usize, pmu_cycle_tmp);
				parent.add_child(pmu_adapter_downstream);










			} else if pmu_sender_vec_tmp[0] == no_connection
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pmu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pmu_receiver_vec_tmp[k]).unwrap());
				}

				let (sender, receiver) = parent.bounded(1024);
				let mut tmp_sender_vec: Vec<dam::channel::Sender<usize>> = vec![];
				tmp_sender_vec.push(sender);
				
				// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, tmp_sender_vec, 1, pmu_cycle_tmp as usize, pmu_cycle_tmp as usize, num_input as usize);
				// parent.add_child(kernel);






				let (wr_addr_sender, wr_addr_receiver) = parent.bounded(1024);
				let (wr_data_sender, wr_data_receiver) = parent.bounded(1024);
				let (ack_sender, ack_receiver) = parent.bounded(1024);
				let (rd_addr_sender, rd_addr_receiver) = parent.bounded(1024);
				let (rd_data_sender, rd_data_receiver) = parent.bounded(1024);

				let pmu_adapter_upstream = pmu_adapter_upstream::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input as usize, pmu_cycle_tmp);
				parent.add_child(pmu_adapter_upstream);

				let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
					num_vec_per_pmu,
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
				let tmp = pmu_cycle_tmp * num_input;
				rd_addr_gen.set_run(move |time| {
					for idx in 0..tmp
					{
						ack_receiver.dequeue(time).unwrap();
						let curr_time = time.tick();
						rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
					}
				});
				parent.add_child(rd_addr_gen);




				
				let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tmp_sender_vec, 1, num_input as usize, pmu_cycle_tmp);
				parent.add_child(pmu_adapter_downstream);









				let con = ConsumerContext::new(receiver);
				parent.add_child(con);


			} else if pmu_receiver_vec_tmp[0] == no_connection && pmu_sender_vec_tmp[0] == no_connection
			{
				panic!("Wrong!");


			} else
			{
				let mut tmp_receiver_vec = vec![];
				for k in 0..pmu_receiver_vec_tmp.len()
				{
					tmp_receiver_vec.push(receiver_map.remove(&pmu_receiver_vec_tmp[k]).unwrap());
				}

				let mut tmp_sender_vec = vec![];
				for k in 0..pmu_sender_vec_tmp.len()
				{
					tmp_sender_vec.push(sender_map.remove(&pmu_sender_vec_tmp[k]).unwrap());
				}


				
				// let kernel: kernel_multi_in_out<_> = kernel_multi_in_out::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, pmu_cycle_tmp as usize, pmu_cycle_tmp as usize, num_input as usize);
				// parent.add_child(kernel);

				



				let (wr_addr_sender, wr_addr_receiver) = parent.bounded(1024);
				let (wr_data_sender, wr_data_receiver) = parent.bounded(1024);
				let (ack_sender, ack_receiver) = parent.bounded(1024);
				let (rd_addr_sender, rd_addr_receiver) = parent.bounded(1024);
				let (rd_data_sender, rd_data_receiver) = parent.bounded(1024);

				let pmu_adapter_upstream = pmu_adapter_upstream::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input as usize, pmu_cycle_tmp);
				parent.add_child(pmu_adapter_upstream);

				let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
					num_vec_per_pmu,
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
				let tmp = pmu_cycle_tmp * num_input;
				rd_addr_gen.set_run(move |time| {
					for idx in 0..tmp
					{
						ack_receiver.dequeue(time).unwrap();
						let curr_time = time.tick();
						rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
					}
				});
				parent.add_child(rd_addr_gen);




				
				let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, num_input as usize, pmu_cycle_tmp);
				parent.add_child(pmu_adapter_downstream);









			}
		}




		if experiment == 3
		{
			println!("experiment 3 ***************************************************************************************");

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


			let time = executed.elapsed_cycles().unwrap();
			let time_tmp: f32 = time as f32 / num_input as f32;
			experiment_time.push(time_tmp as usize);
		}






















		println!("\n\n\n\n\n\n\n");	
	}

	println!("dfmodel_time {:?}", dfmodel_time);
	println!("experiment_time {:?}", experiment_time);
}


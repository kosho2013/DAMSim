extern crate protobuf;

mod proto_driver;
pub mod templates;
pub mod utils;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::{env, mem, process};
use std::{fs, time::Instant};
use std::fs::{read_to_string, File};
use std::io::{self, BufRead, BufReader};
use dam::channel::{ChannelElement, Receiver};
use dam::templates::datastore::Behavior;
use dam::templates::pmu::{PMUReadBundle, PMUWriteBundle, PMU};
use dam::types::StaticallySized;
use dam::utility_contexts::{ConsumerContext, FunctionContext, GeneratorContext};
use proto_driver::proto_headers::setup::System;
use prost::Message;
use dam::{logging::LogEvent, simulation::*};
use templates::kernel::kernel;
use templates::kernel_multi_in_out::kernel_multi_in_out;
use templates::my_pcu::{make_simd_pcu, make_systolic_pcu};
use templates::pcu_adapter::{simd_pcu_adapter_downstream, simd_pcu_adapter_upstream, systolic_pcu_adapter_downstream, systolic_pcu_adapter_upstream};
use templates::pmu_adapter::{pmu_adapter_downstream, pmu_adapter_upstream};
use templates::router_mesh::router_mesh;
use templates::router_sho_mesh::router_sho_mesh;
use templates::router_sn_mesh::router_sn_mesh;
use templates::router_adapter::{from_router_adapter, to_router_adapter};

fn main() {
	let invalid = 999999;
	let dummy = 1;

	let args: Vec<String> = env::args().collect();
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

	let accelerator = system.accelerator.unwrap();

	let x_dim: usize = accelerator.x_dim as usize;
	let y_dim: usize = accelerator.y_dim as usize;
	let noc_topology: String = accelerator.noc_topology as String;

	let num_compute_tile: usize = accelerator.num_compute_tile as usize;
	let num_memory_tile: usize = accelerator.num_memory_tile as usize;
	let lane_dim: usize = accelerator.lane_dim as usize;
	let stage_dim: usize = accelerator.stage_dim as usize;
	let freq: f32 = accelerator.freq as f32;
	let word: usize = accelerator.word as usize;
	let sram_cap: usize = accelerator.sram_cap as usize;
	let num_switch: usize = accelerator.num_switch as usize;
	let num_vc: usize = accelerator.num_vc as usize;
	let buffer_depth: usize = accelerator.buffer_depth as usize;
	let dram_bw: f32 = accelerator.dram_bw as f32;
	let net_bw: f32 = accelerator.net_bw as f32;

	let num_vec_per_pmu = sram_cap / lane_dim / word;

	let mut Compute_Latency = vec![];
	let mut Memory_Latency = vec![];
	let mut Network_Latency = vec![];
	let mut num_tile: usize = 0;
	
	let log_path = format!("{}{}", &args[1], "/log.txt");
	let lines = read_to_string(log_path).unwrap();


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
		if line.starts_with("Compute_Latency[") 
		{ 
			let tmp: f32 = line.split_whitespace().last().unwrap().parse().unwrap();
			let tmp2 = tmp / num_tile as f32;
			let tmp3: usize = tmp2.round() as usize;
			Compute_Latency.push(tmp3);
		}

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





	
	println!("Compute_Latency {:?}", Compute_Latency);
	println!("Memory_Latency {:?}", Memory_Latency);
	println!("Network_Latency {:?}", Network_Latency);
	println!("num_tile {}", num_tile);
	println!("num_input {}", num_input);
	println!("num_config {}", num_config);


	



	for i in 0..num_config
	{
		println!("config: {}", i);


		let mut connection_first_type: Vec<String> = vec![];
		let mut connection_first_x: Vec<usize> = vec![];
		let mut connection_first_y: Vec<usize> = vec![];
		let mut connection_second_type: Vec<String> = vec![];
		let mut connection_second_x: Vec<usize> = vec![];
		let mut connection_second_y: Vec<usize> = vec![];



		for line in lines.lines() {
			let str = format!("connection config {}", i);
			if line.starts_with(&str)
			{
				let tmp: Vec<&str> = line.split_whitespace().collect();
				let tmp1 = tmp[3].parse().unwrap();
				let tmp2 = tmp[4].parse().unwrap();
				let tmp3 = tmp[5].parse().unwrap();
				let tmp4 = tmp[6].parse().unwrap();
				let tmp5 = tmp[7].parse().unwrap();
				let tmp6 = tmp[8].parse().unwrap();
				connection_first_type.push(tmp1);
				connection_first_x.push(tmp2);
				connection_first_y.push(tmp3);
				connection_second_type.push(tmp4);
				connection_second_x.push(tmp5);
				connection_second_y.push(tmp6);
			}	
		}



		



		let mut pcu_x: Vec<usize> = vec![];
		let mut pcu_y: Vec<usize> = vec![];
		let mut pcu_counter: Vec<usize> = vec![];
		let mut pcu_sender_vec: Vec<Vec<usize>> = vec![];
		let mut pcu_receiver_vec: Vec<Vec<usize>> = vec![];

		let mut pcu_SIMD_or_Systolic: Vec<&str> = vec![];
		let mut pcu_M: Vec<usize> = vec![];
		let mut pcu_K: Vec<usize> = vec![];
		let mut pcu_N: Vec<usize> = vec![];
		

		let mut pmu_x: Vec<usize> = vec![];
		let mut pmu_y: Vec<usize> = vec![];
		let mut pmu_counter: Vec<usize> = vec![];
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
				pcu_counter.push(tmp[6].parse().unwrap());
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
				pmu_counter.push(tmp[6].parse().unwrap());

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
		println!("pcu_counter {:?}", pcu_counter);
		println!("pcu_sender_vec {:?}", pcu_sender_vec);
		println!("pcu_receiver_vec {:?}", pcu_receiver_vec);

		println!("pcu_SIMD_or_Systolic {:?}", pcu_SIMD_or_Systolic);
		println!("pcu_M {:?}", pcu_M);
		println!("pcu_K {:?}", pcu_K);
		println!("pcu_N {:?}", pcu_N);

		println!("pmu_x{:?}", pmu_x);
		println!("pmu_y{:?}", pmu_y);
		println!("pmu_counter {:?}", pmu_counter);
		println!("pmu_sender_vec {:?}", pmu_sender_vec);
		println!("pmu_receiver_vec {:?}", pmu_receiver_vec);

		println!("num_of_connections {}", num_of_connections);

		println!("\n\n\n\n");








		if experiment == 1
		{
			println!("experiment 1, perfect NoC ***************************************************************************************");

			let mut parent = ProgramBuilder::default();

			// DRAM
			// let mut sender_map_mem: HashMap<usize, dam::channel::Sender<usize>> = HashMap::new();
			// let mut receiver_map_mem: HashMap<usize, dam::channel::Receiver<usize>> = HashMap::new();
			// for j in 0..2
			// {
			// 	let (sender, receiver) = parent.bounded(buffer_depth);
			// 	sender_map_mem.insert(j, sender);
			// 	receiver_map_mem.insert(j, receiver);
			// }


			// let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
			// let input = GeneratorContext::new(iter, sender_map_mem.remove(&0).unwrap());
			// parent.add_child(input);

			// let memory = kernel::new(receiver_map_mem.remove(&0).unwrap(), sender_map_mem.remove(&1).unwrap(), Memory_Latency[i] as usize, Memory_Latency[i] as usize, num_input as usize, dummy);
			// parent.add_child(memory);

			// let output: ConsumerContext<usize> = ConsumerContext::new(receiver_map_mem.remove(&1).unwrap());
			// parent.add_child(output);



			// network
			// let mut sender_map_net: HashMap<usize, dam::channel::Sender<usize>> = HashMap::new();
			// let mut receiver_map_net: HashMap<usize, dam::channel::Receiver<usize>> = HashMap::new();
			// for j in 0..2
			// {
			// 	let (sender, receiver) = parent.bounded(buffer_depth);
			// 	sender_map_net.insert(j, sender);
			// 	receiver_map_net.insert(j, receiver);
			// }

			// let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
			// let input = GeneratorContext::new(iter, sender_map_net.remove(&0).unwrap());
			// parent.add_child(input);

			// let network = kernel::new(receiver_map_net.remove(&0).unwrap(), sender_map_net.remove(&1).unwrap(), Network_Latency[i] as usize, Network_Latency[i] as usize, num_input as usize, dummy);
			// parent.add_child(network);

			// let output: ConsumerContext<usize> = ConsumerContext::new(receiver_map_net.remove(&1).unwrap());
			// parent.add_child(output);





			let mut sender_map: HashMap<usize, dam::channel::Sender<usize>> = HashMap::new();
			let mut receiver_map: HashMap<usize, dam::channel::Receiver<usize>> = HashMap::new();
			for j in 0..num_of_connections
			{
				let (sender, receiver) = parent.bounded(buffer_depth);
				sender_map.insert(j, sender);
				receiver_map.insert(j, receiver);
			}





			// compute tile
			for j in 0..pcu_x.len()
			{
				let mut pcu_x_tmp = pcu_x[j];
				let mut pcu_y_tmp = pcu_y[j];
				let mut pcu_counter_tmp = pcu_counter[j];
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



				let no_connection = invalid;
				if pcu_receiver_vec_tmp[0] == no_connection
				{
					let mut tmp_sender_vec = vec![];
					let mut tmp_dst_vec = vec![];
					for k in 0..pcu_sender_vec_tmp.len()
					{
						let mut connection_id = pcu_sender_vec_tmp[k];
						tmp_sender_vec.push(sender_map.remove(&connection_id).unwrap());

						if connection_first_type[connection_id] == "pcu" && connection_first_x[connection_id] == pcu_x_tmp && connection_first_y[connection_id] == pcu_y_tmp
						{
							tmp_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
						} else {
							panic!("Wrong!");
						}
					}

					let (sender, receiver) = parent.bounded(buffer_depth);
					let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
					let gen = GeneratorContext::new(iter, sender);
					parent.add_child(gen);


					let mut tmp_receiver_vec = vec![];
					tmp_receiver_vec.push(receiver);
					


					if simd_or_systolic == "SIMD"
					{					
						let (sender1, receiver1) = parent.bounded(buffer_depth);
						let (sender2, receiver2) = parent.bounded(buffer_depth);
						
						let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize, dummy);
						parent.add_child(simd_pcu_adapter_upstream);

						let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
						parent.add_child(pcu);

						let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, M as usize, K as usize, N as usize, dummy);
						parent.add_child(simd_pcu_adapter_downstream);

					} else if simd_or_systolic == "Systolic"
					{

						let (sender1, receiver1) = parent.bounded(buffer_depth);
						let (sender2, receiver2) = parent.bounded(buffer_depth);
						let (sender3, receiver3) = parent.bounded(buffer_depth);
						let (sender4, receiver4) = parent.bounded(buffer_depth);

						let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
						parent.add_child(systolic_pcu_adapter_upstream);

						let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
						parent.add_child(pcu_lane);

						let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
						parent.add_child(pcu_stage);

						let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
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
					let mut tmp_dst_vec = vec![];
					tmp_dst_vec.push(no_connection);

					let (sender, receiver) = parent.bounded(buffer_depth);
					let mut tmp_sender_vec: Vec<dam::channel::Sender<usize>> = vec![];
					tmp_sender_vec.push(sender);
					
					


					if simd_or_systolic == "SIMD"
					{					
						let (sender1, receiver1) = parent.bounded(buffer_depth);
						let (sender2, receiver2) = parent.bounded(buffer_depth);
						
						let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize, dummy);
						parent.add_child(simd_pcu_adapter_upstream);

						let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
						parent.add_child(pcu);

						let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, M as usize, K as usize, N as usize, dummy);
						parent.add_child(simd_pcu_adapter_downstream);

					} else if simd_or_systolic == "Systolic"
					{

						let (sender1, receiver1) = parent.bounded(buffer_depth);
						let (sender2, receiver2) = parent.bounded(buffer_depth);
						let (sender3, receiver3) = parent.bounded(buffer_depth);
						let (sender4, receiver4) = parent.bounded(buffer_depth);

						let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
						parent.add_child(systolic_pcu_adapter_upstream);

						let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
						parent.add_child(pcu_lane);

						let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
						parent.add_child(pcu_stage);

						let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
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
					let mut tmp_dst_vec = vec![];
					for k in 0..pcu_sender_vec_tmp.len()
					{
						let mut connection_id = pcu_sender_vec_tmp[k];
						tmp_sender_vec.push(sender_map.remove(&connection_id).unwrap());

						if connection_first_type[connection_id] == "pcu" && connection_first_x[connection_id] == pcu_x_tmp && connection_first_y[connection_id] == pcu_y_tmp
						{
							tmp_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
						} else {
							panic!("Wrong!");
						}
					}



					if simd_or_systolic == "SIMD"
					{					
						let (sender1, receiver1) = parent.bounded(buffer_depth);
						let (sender2, receiver2) = parent.bounded(buffer_depth);
						
						let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize, dummy);
						parent.add_child(simd_pcu_adapter_upstream);

						let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
						parent.add_child(pcu);

						let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, M as usize, K as usize, N as usize, dummy);
						parent.add_child(simd_pcu_adapter_downstream);

					} else if simd_or_systolic == "Systolic"
					{

						let (sender1, receiver1) = parent.bounded(buffer_depth);
						let (sender2, receiver2) = parent.bounded(buffer_depth);
						let (sender3, receiver3) = parent.bounded(buffer_depth);
						let (sender4, receiver4) = parent.bounded(buffer_depth);

						let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tmp_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
						parent.add_child(systolic_pcu_adapter_upstream);

						let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
						parent.add_child(pcu_lane);

						let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
						parent.add_child(pcu_stage);

						let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tmp_sender_vec, pcu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
						parent.add_child(systolic_pcu_adapter_downstream);









					} else {
						panic!("Wrong!");
					}


				}
			}







			// memory tile
			for j in 0..pmu_x.len()
			{
				let mut pmu_x_tmp = pmu_x[j];
				let mut pmu_y_tmp = pmu_y[j];
				let mut pmu_counter_tmp = pmu_counter[j];
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
				
				let no_connection = invalid;
				if pmu_receiver_vec_tmp[0] == no_connection
				{
					let mut tmp_sender_vec = vec![];
					let mut tmp_dst_vec = vec![];
					for k in 0..pmu_sender_vec_tmp.len()
					{
						let mut connection_id = pmu_sender_vec_tmp[k];
						tmp_sender_vec.push(sender_map.remove(&connection_id).unwrap());

						if connection_first_type[connection_id] == "pmu" && connection_first_x[connection_id] == pmu_x_tmp && connection_first_y[connection_id] == pmu_y_tmp
						{
							tmp_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
						} else {
							panic!("Wrong!");
						}


					}
					let (sender, receiver) = parent.bounded(buffer_depth);
					let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
					let gen = GeneratorContext::new(iter, sender);
					parent.add_child(gen);


					let mut tmp_receiver_vec = vec![];
					tmp_receiver_vec.push(receiver);
					



					let (wr_addr_sender, wr_addr_receiver) = parent.bounded(buffer_depth);
					let (wr_data_sender, wr_data_receiver) = parent.bounded(buffer_depth);
					let (ack_sender, ack_receiver) = parent.bounded(buffer_depth);
					let (rd_addr_sender, rd_addr_receiver) = parent.bounded(buffer_depth);
					let (rd_data_sender, rd_data_receiver) = parent.bounded(buffer_depth);

					let pmu_adapter_upstream = pmu_adapter_upstream::new(tmp_receiver_vec, 1, wr_addr_sender, wr_data_sender, num_input as usize, pmu_counter_tmp, dummy);
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
					let tmp = pmu_counter_tmp * num_input;
					rd_addr_gen.set_run(move |time| {
						for idx in 0..tmp
						{
							ack_receiver.dequeue(time).unwrap();
							let curr_time = time.tick();
							rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
						}
					});
					parent.add_child(rd_addr_gen);




					
					let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, pmu_counter_tmp, dummy);
					parent.add_child(pmu_adapter_downstream);










				} else if pmu_sender_vec_tmp[0] == no_connection
				{
					let mut tmp_receiver_vec = vec![];
					for k in 0..pmu_receiver_vec_tmp.len()
					{
						tmp_receiver_vec.push(receiver_map.remove(&pmu_receiver_vec_tmp[k]).unwrap());
					}

					let (sender, receiver) = parent.bounded(buffer_depth);
					let mut tmp_sender_vec: Vec<dam::channel::Sender<usize>> = vec![];
					tmp_sender_vec.push(sender);

					let mut tmp_dst_vec = vec![];
					tmp_dst_vec.push(no_connection);

					




					let (wr_addr_sender, wr_addr_receiver) = parent.bounded(buffer_depth);
					let (wr_data_sender, wr_data_receiver) = parent.bounded(buffer_depth);
					let (ack_sender, ack_receiver) = parent.bounded(buffer_depth);
					let (rd_addr_sender, rd_addr_receiver) = parent.bounded(buffer_depth);
					let (rd_data_sender, rd_data_receiver) = parent.bounded(buffer_depth);

					let pmu_adapter_upstream = pmu_adapter_upstream::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input as usize, pmu_counter_tmp, dummy);
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
					let tmp = pmu_counter_tmp * num_input;
					rd_addr_gen.set_run(move |time| {
						for idx in 0..tmp
						{
							ack_receiver.dequeue(time).unwrap();
							let curr_time = time.tick();
							rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
						}
					});
					parent.add_child(rd_addr_gen);




					
					let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tmp_sender_vec, 1, tmp_dst_vec, num_input as usize, pmu_counter_tmp, dummy);
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
					let mut tmp_dst_vec = vec![];
					for k in 0..pmu_sender_vec_tmp.len()
					{
						let mut connection_id = pmu_sender_vec_tmp[k];
						tmp_sender_vec.push(sender_map.remove(&connection_id).unwrap());

						if connection_first_type[connection_id] == "pmu" && connection_first_x[connection_id] == pmu_x_tmp && connection_first_y[connection_id] == pmu_y_tmp
						{
							tmp_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
						} else {
							panic!("Wrong!");
						}
					}



					let (wr_addr_sender, wr_addr_receiver) = parent.bounded(buffer_depth);
					let (wr_data_sender, wr_data_receiver) = parent.bounded(buffer_depth);
					let (ack_sender, ack_receiver) = parent.bounded(buffer_depth);
					let (rd_addr_sender, rd_addr_receiver) = parent.bounded(buffer_depth);
					let (rd_data_sender, rd_data_receiver) = parent.bounded(buffer_depth);

					let pmu_adapter_upstream = pmu_adapter_upstream::new(tmp_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input as usize, pmu_counter_tmp, dummy);
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
					let tmp = pmu_counter_tmp * num_input;
					rd_addr_gen.set_run(move |time| {
						for idx in 0..tmp
						{
							ack_receiver.dequeue(time).unwrap();
							let curr_time = time.tick();
							rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
						}
					});
					parent.add_child(rd_addr_gen);




					
					let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tmp_sender_vec, pmu_sender_vec_tmp.len() as usize, tmp_dst_vec, num_input as usize, pmu_counter_tmp, dummy);
					parent.add_child(pmu_adapter_downstream);









				}
			}




		
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


			let dam_compute_time = executed.elapsed_cycles().unwrap();
			let dam_compute_time: f32 = dam_compute_time as f32 / num_input as f32 / freq as f32;
			println!("DFModel compute latency per tile {}", Compute_Latency[i]);
			println!("DFModel memory latency per tile {}", Memory_Latency[i]);
			println!("DFModel network latency per tile {}", Network_Latency[i]);
			println!("DFModel overall latency per tile {}", dfmodel_time[i]);
			println!("DAM compute latency per tile {}", dam_compute_time);
		}


























		if experiment == 2
		{
			println!("experiment 2, real NoC ***************************************************************************************");

			let mut parent = ProgramBuilder::default();
			
			
			
			let mut used_link_map = HashMap::new();

			if noc_topology == "mesh"
			{
				// get which global NoC is used for routing
				for j in 0..connection_first_x.len()
				{
					let mut curr_x = connection_first_x[j];
					let mut curr_y = connection_first_y[j];
					let mut dst_x = connection_second_x[j];
					let mut dst_y = connection_second_y[j];

					let link = (curr_x, curr_y, "from_L".to_owned(), curr_x, curr_y, "from_L".to_owned());
					if used_link_map.contains_key(&link)
					{	
						let tmp = used_link_map[&link] + 1;
						used_link_map.insert(link, tmp);
					} else
					{ 
						used_link_map.insert(link, 1);
					}


					let link = (dst_x, dst_y, "to_L".to_owned(), dst_x, dst_y, "to_L".to_owned());
					if used_link_map.contains_key(&link)
					{	
						let tmp = used_link_map[&link] + 1;
						used_link_map.insert(link, tmp);
					} else
					{ 
						used_link_map.insert(link, 1);
					}


					while true
					{
						if dst_x == curr_x && dst_y == curr_y // exit local port
						{
							break;
						} else if dst_x == curr_x && dst_y < curr_y // exit W port
						{
							let link = (curr_x, curr_y, "W".to_owned(), curr_x, curr_y-1, "E".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y -= 1;

						} else if dst_x < curr_x && dst_y < curr_y // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;

						} else if dst_x < curr_x && dst_y == curr_y // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;

						} else if dst_x < curr_x && dst_y > curr_y // exit N port
						{
							let link = (curr_x, curr_y, "N".to_owned(), curr_x-1, curr_y, "S".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x -= 1;

						} else if dst_x == curr_x && dst_y > curr_y // exit E port
						{
							let link = (curr_x, curr_y, "E".to_owned(), curr_x, curr_y+1, "W".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_y += 1

						} else if dst_x > curr_x && dst_y > curr_y // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;

						} else if dst_x > curr_x && dst_y == curr_y // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;

						} else if dst_x > curr_x && dst_y < curr_y // exit S port
						{
							let link = (curr_x, curr_y, "S".to_owned(), curr_x+1, curr_y, "N".to_owned());
							if used_link_map.contains_key(&link)
							{	
								let tmp = used_link_map[&link] + 1;
								used_link_map.insert(link, tmp);
							} else
							{ 
								used_link_map.insert(link, 1);
							}
							curr_x += 1;

						} else
						{
							panic!("Wrong!");
						}
					}
				}

			} else if noc_topology == "sho_mesh"
			{

			} else if noc_topology == "sn_mesh"
			{

			} else
			{
				panic!("Wrong!");
			}


			










			




			// NoC global links
			let mut sender_map_noc_global: HashMap<(usize, usize, String, usize, usize, String), dam::channel::Sender<usize>> = HashMap::new();
			let mut receiver_map_noc_global: HashMap<(usize, usize, String, usize, usize, String), dam::channel::Receiver<usize>> = HashMap::new();

			for ele in used_link_map.keys()
			{
				if ele.2 == "to_L".to_owned() || ele.2 == "from_L".to_owned()
				{

				} else
				{
					let (sender, receiver) = parent.bounded(buffer_depth);
					sender_map_noc_global.insert(ele.clone(), sender);
					receiver_map_noc_global.insert(ele.clone(), receiver);
				}
			}

			println!("used_link_map {:?}", used_link_map);










			// all involved routers
			let mut all_routers = HashSet::new();
			for ele in sender_map_noc_global.keys()
			{
				all_routers.insert((ele.0, ele.1));
				all_routers.insert((ele.3, ele.4));
			}

			for ele in receiver_map_noc_global.keys()
			{
				all_routers.insert((ele.0, ele.1));
				all_routers.insert((ele.3, ele.4));
			}

			println!("all_routers{:?}", all_routers);


			// extra routers, not attached to local PCUs/PMUs
			let mut extra_routers = HashSet::new();
			for ele in all_routers
			{
				let mut flag = false;
				for j in 0..pcu_x.len()
				{
					if pcu_x[j] == ele.0 && pcu_y[j] == ele.1
					{
						flag = true;
					}
				}
				for j in 0..pmu_x.len()
				{
					if pmu_x[j] == ele.0 && pmu_y[j] == ele.1
					{
						flag = true;
					}
				}

				if !flag
				{
					extra_routers.insert(ele.clone());
				}
			}


			
			println!("extra_routers{:?}", extra_routers);



			

			// DRAM
			// let mut sender_map_mem: HashMap<usize, dam::channel::Sender<usize>> = HashMap::new();
			// let mut receiver_map_mem: HashMap<usize, dam::channel::Receiver<usize>> = HashMap::new();
			// for j in 0..2
			// {
			// 	let (sender, receiver) = parent.bounded(buffer_depth);
			// 	sender_map_mem.insert(j, sender);
			// 	receiver_map_mem.insert(j, receiver);
			// }


			// let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
			// let input = GeneratorContext::new(iter, sender_map_mem.remove(&0).unwrap());
			// parent.add_child(input);

			// let memory = kernel::new(receiver_map_mem.remove(&0).unwrap(), sender_map_mem.remove(&1).unwrap(), Memory_Latency[i] as usize, Memory_Latency[i] as usize, num_input as usize, dummy);
			// parent.add_child(memory);

			// let output: ConsumerContext<usize> = ConsumerContext::new(receiver_map_mem.remove(&1).unwrap());
			// parent.add_child(output);



			// network
			// let mut sender_map_net: HashMap<usize, dam::channel::Sender<usize>> = HashMap::new();
			// let mut receiver_map_net: HashMap<usize, dam::channel::Receiver<usize>> = HashMap::new();
			// for j in 0..2
			// {
			// 	let (sender, receiver) = parent.bounded(buffer_depth);
			// 	sender_map_net.insert(j, sender);
			// 	receiver_map_net.insert(j, receiver);
			// }

			// let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
			// let input = GeneratorContext::new(iter, sender_map_net.remove(&0).unwrap());
			// parent.add_child(input);

			// let network = kernel::new(receiver_map_net.remove(&0).unwrap(), sender_map_net.remove(&1).unwrap(), Network_Latency[i] as usize, Network_Latency[i] as usize, num_input as usize, dummy);
			// parent.add_child(network);

			// let output: ConsumerContext<usize> = ConsumerContext::new(receiver_map_net.remove(&1).unwrap());
			// parent.add_child(output);











			




			// compute tile
			for x in 0..x_dim
			{	
				for y in 0..y_dim
				{
					for j in 0..pcu_x.len()
					{
						if pcu_x[j] == x && pcu_y[j] == y
						{
							// router setup
							let mut router_in_stream = vec![];
							let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
							let mut router_in_len = 0;
							
							let mut router_out_stream = vec![];
							let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
							let mut router_out_len = 0;


							




							if noc_topology == "mesh"
							{
								// global links
								if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
								{
									let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
									router_in_stream.push(N_in);
									
									let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
									router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
								{
									let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
									router_in_stream.push(S_in);

									let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
									router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
								{
									let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
									router_in_stream.push(E_in);

									let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
									router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
								{
									let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
									router_in_stream.push(W_in);

									let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
									router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
								{
									let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
									router_out_stream.push(N_out);

									let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
									router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
								{
									let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
									router_out_stream.push(S_out);

									let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
									router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
								{
									let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
									router_out_stream.push(E_out);

									let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
									router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
								{
									let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
									router_out_stream.push(W_out);

									let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
									router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
							} else if noc_topology == "sho_mesh"
							{

							} else if noc_topology == "sn_mesh"
							{

							} else
							{
								panic!("Wrong!");
							}

							



							
							





							let mut pcu_counter_tmp = pcu_counter[j];
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



							let no_connection = invalid;
							if pcu_receiver_vec_tmp[0] == no_connection
							{
								let mut tile_receiver_vec = vec![];
								let mut tile_sender_vec = vec![];
								let mut tile_dst_vec = vec![];
								let mut router_receiver_vec = vec![];

								for k in 0..pcu_sender_vec_tmp.len()
								{
									let (sender, receiver) = parent.bounded(buffer_depth);
									tile_sender_vec.push(sender);
									router_receiver_vec.push(receiver);

									let mut connection_id = pcu_sender_vec_tmp[k];
									if connection_first_type[connection_id] == "pcu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
									{
										tile_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
									} else {
										panic!("Wrong!");
									}
								}
								
								let (sender, receiver) = parent.bounded(buffer_depth);
								let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
								let gen = GeneratorContext::new(iter, sender);
								parent.add_child(gen);
								tile_receiver_vec.push(receiver);

								if simd_or_systolic == "SIMD"
								{					
									let (sender1, receiver1) = parent.bounded(buffer_depth);
									let (sender2, receiver2) = parent.bounded(buffer_depth);
									
									let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tile_receiver_vec, 1, sender1, num_input as usize, M as usize, K as usize, N as usize, dummy);
									parent.add_child(simd_pcu_adapter_upstream);

									let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
									parent.add_child(pcu);

									let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input as usize, M as usize, K as usize, N as usize, dummy);
									parent.add_child(simd_pcu_adapter_downstream);
								} else if simd_or_systolic == "Systolic"
								{
									let (sender1, receiver1) = parent.bounded(buffer_depth);
									let (sender2, receiver2) = parent.bounded(buffer_depth);
									let (sender3, receiver3) = parent.bounded(buffer_depth);
									let (sender4, receiver4) = parent.bounded(buffer_depth);

									let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tile_receiver_vec, 1, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
									parent.add_child(systolic_pcu_adapter_upstream);

									let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
									parent.add_child(pcu_lane);

									let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
									parent.add_child(pcu_stage);

									let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
									parent.add_child(systolic_pcu_adapter_downstream);
								} else {
									panic!("Wrong!");
								}


								let (sender, receiver) = parent.bounded(buffer_depth);
								let to_router_adapter = to_router_adapter::new(router_receiver_vec, pcu_sender_vec_tmp.len(), sender, num_input, dummy);
								parent.add_child(to_router_adapter);

								router_in_stream.push(receiver);


								let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
								router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;



								println!("PCU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
								if noc_topology == "mesh"
								{
									let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
									parent.add_child(router_mesh);
								} else if noc_topology == "sho_mesh"
								{

								} else if noc_topology == "sn_mesh"
								{

								} else
								{
									panic!("Wrong!");
								}

								

							} else if pcu_sender_vec_tmp[0] == no_connection
							{
								// let mut router_sender_vec = vec![];
								// let mut tile_receiver_vec = vec![];
								// let mut tile_sender_vec = vec![];
								// let mut tile_dst_vec = vec![];
								// for k in 0..pcu_receiver_vec_tmp.len()
								// {
								// 	let (sender, receiver) = parent.bounded(buffer_depth);
								// 	router_sender_vec.push(sender);
								// 	tile_receiver_vec.push(receiver);
								// }
								// tile_dst_vec.push(no_connection);

								// let (sender, receiver) = parent.bounded(buffer_depth);
								// tile_sender_vec.push(sender);
								
								


								// if simd_or_systolic == "SIMD"
								// {					
								// 	let (sender1, receiver1) = parent.bounded(buffer_depth);
								// 	let (sender2, receiver2) = parent.bounded(buffer_depth);
									
								// 	let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tile_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize, dummy);
								// 	parent.add_child(simd_pcu_adapter_upstream);

								// 	let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
								// 	parent.add_child(pcu);

								// 	let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tile_sender_vec, 1, tile_dst_vec, num_input as usize, M as usize, K as usize, N as usize, dummy);
								// 	parent.add_child(simd_pcu_adapter_downstream);
								// } else if simd_or_systolic == "Systolic"
								// {
								// 	let (sender1, receiver1) = parent.bounded(buffer_depth);
								// 	let (sender2, receiver2) = parent.bounded(buffer_depth);
								// 	let (sender3, receiver3) = parent.bounded(buffer_depth);
								// 	let (sender4, receiver4) = parent.bounded(buffer_depth);

								// 	let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tile_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
								// 	parent.add_child(systolic_pcu_adapter_upstream);

								// 	let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
								// 	parent.add_child(pcu_lane);

								// 	let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
								// 	parent.add_child(pcu_stage);

								// 	let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tile_sender_vec, 1, tile_dst_vec, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
								// 	parent.add_child(systolic_pcu_adapter_downstream);
								// } else {
								// 	panic!("Wrong!");
								// }

								





								let (sender, receiver) = parent.bounded(buffer_depth);
								router_out_stream.push(sender);

								let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
								router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;


								println!("PCU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
								if noc_topology == "mesh"
								{
									let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
									parent.add_child(router_mesh);
								} else if noc_topology == "sho_mesh"
								{

								} else if noc_topology == "sn_mesh"
								{

								} else
								{
									panic!("Wrong!");
								}



								// let from_router_adapter = from_router_adapter::new(receiver, router_sender_vec, pcu_receiver_vec_tmp.len(), num_input, dummy);
								// parent.add_child(from_router_adapter);


								let con = ConsumerContext::new(receiver);
								parent.add_child(con);






							} else if pcu_receiver_vec_tmp[0] == no_connection && pcu_sender_vec_tmp[0] == no_connection
							{
								panic!("Wrong!");


							} else
							{
								let mut tile_sender_vec = vec![];
								let mut tile_dst_vec = vec![];
								let mut router_receiver_vec = vec![];

								let mut router_sender_vec = vec![];
								let mut tile_receiver_vec = vec![];

								for k in 0..pcu_receiver_vec_tmp.len()
								{
									let (sender, receiver) = parent.bounded(buffer_depth);
									router_sender_vec.push(sender);
									tile_receiver_vec.push(receiver);
								}
								for k in 0..pcu_sender_vec_tmp.len()
								{
									let (sender, receiver) = parent.bounded(buffer_depth);
									tile_sender_vec.push(sender);
									router_receiver_vec.push(receiver);

									let mut connection_id = pcu_sender_vec_tmp[k];
									if connection_first_type[connection_id] == "pcu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
									{
										tile_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
									} else {
										panic!("Wrong!");
									}
								}



								if simd_or_systolic == "SIMD"
								{					
									let (sender1, receiver1) = parent.bounded(buffer_depth);
									let (sender2, receiver2) = parent.bounded(buffer_depth);
									
									let simd_pcu_adapter_upstream = simd_pcu_adapter_upstream::new(tile_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, num_input as usize, M as usize, K as usize, N as usize, dummy);
									parent.add_child(simd_pcu_adapter_upstream);

									let pcu = make_simd_pcu(stage_dim, receiver1, sender2);
									parent.add_child(pcu);

									let simd_pcu_adapter_downstream = simd_pcu_adapter_downstream::new(receiver2, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input as usize, M as usize, K as usize, N as usize, dummy);
									parent.add_child(simd_pcu_adapter_downstream);
								} else if simd_or_systolic == "Systolic"
								{
									let (sender1, receiver1) = parent.bounded(buffer_depth);
									let (sender2, receiver2) = parent.bounded(buffer_depth);
									let (sender3, receiver3) = parent.bounded(buffer_depth);
									let (sender4, receiver4) = parent.bounded(buffer_depth);

									let systolic_pcu_adapter_upstream = systolic_pcu_adapter_upstream::new(tile_receiver_vec, pcu_receiver_vec_tmp.len() as usize, sender1, sender2, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
									parent.add_child(systolic_pcu_adapter_upstream);

									let pcu_lane = make_systolic_pcu(stage_dim, receiver1, sender3);
									parent.add_child(pcu_lane);

									let pcu_stage = make_systolic_pcu(lane_dim, receiver2, sender4);
									parent.add_child(pcu_stage);

									let systolic_pcu_adapter_downstream = systolic_pcu_adapter_downstream::new(receiver3, receiver4, tile_sender_vec, pcu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input as usize, M as usize, K as usize, N as usize, lane_dim, stage_dim, dummy);
									parent.add_child(systolic_pcu_adapter_downstream);
								} else {
									panic!("Wrong!");
								}




								let (sender1, receiver1) = parent.bounded(buffer_depth);
								let (sender2, receiver2) = parent.bounded(buffer_depth);

								router_in_stream.push(receiver1);

								let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
								router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;


								router_out_stream.push(sender2);

								let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
								router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;




								let to_router_adapter = to_router_adapter::new(router_receiver_vec, pcu_sender_vec_tmp.len(), sender1, num_input, dummy);
								parent.add_child(to_router_adapter);

								println!("PCU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
								if noc_topology == "mesh"
								{
									let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
									parent.add_child(router_mesh);
								} else if noc_topology == "sho_mesh"
								{

								} else if noc_topology == "sn_mesh"
								{

								} else
								{
									panic!("Wrong!");
								}



								let from_router_adapter = from_router_adapter::new(receiver2, router_sender_vec, pcu_receiver_vec_tmp.len(), num_input, dummy);
								parent.add_child(from_router_adapter);

							}
						}
					}
				}
			}







			// memory tile
			for x in 0..x_dim
			{	
				for y in 0..y_dim
				{
					for j in 0..pmu_x.len()
					{
						if pmu_x[j] == x && pmu_y[j] == y
						{
							let mut x = pmu_x[j];
							let mut y = pmu_y[j];
							
							// router setup
							let mut router_in_stream = vec![];
							let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
							let mut router_in_len = 0;
							
							let mut router_out_stream = vec![];
							let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
							let mut router_out_len = 0;





							if noc_topology == "mesh"
							{
								// global links
								if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
								{
									let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
									router_in_stream.push(N_in);
									
									let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
									router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
								{
									let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
									router_in_stream.push(S_in);

									let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
									router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
								{
									let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
									router_in_stream.push(E_in);

									let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
									router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
								{
									let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
									router_in_stream.push(W_in);

									let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
									router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
									router_in_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
								{
									let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
									router_out_stream.push(N_out);

									let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
									router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
								{
									let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
									router_out_stream.push(S_out);

									let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
									router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
								{
									let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
									router_out_stream.push(E_out);

									let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
									router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
								
								if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
								{
									let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
									router_out_stream.push(W_out);

									let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
									router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
									router_out_len += 1;
								}
							} else if noc_topology == "sho_mesh"
							{

							} else if noc_topology == "sn_mesh"
							{

							} else
							{
								panic!("Wrong!");
							}



							







							let mut pmu_counter_tmp = pmu_counter[j];
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
							



							let no_connection = invalid;
							if pmu_receiver_vec_tmp[0] == no_connection
							{
								
								
								
								let mut tile_receiver_vec = vec![];
								let mut tile_sender_vec = vec![];
								let mut tile_dst_vec = vec![];
								let mut router_receiver_vec = vec![];




								for k in 0..pmu_sender_vec_tmp.len()
								{
									let (sender, receiver) = parent.bounded(buffer_depth);
									tile_sender_vec.push(sender);
									router_receiver_vec.push(receiver);

									let mut connection_id = pmu_sender_vec_tmp[k];
									if connection_first_type[connection_id] == "pmu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
									{
										tile_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
									} else {
										panic!("Wrong!");
									}
								}

								
								let (sender, receiver) = parent.bounded(buffer_depth);
								let iter = || (0..(num_input)).map(|i| (i as usize) * 1_usize);
								let gen = GeneratorContext::new(iter, sender);
								parent.add_child(gen);
								tile_receiver_vec.push(receiver);








								// PMU setup
								let (wr_addr_sender, wr_addr_receiver) = parent.bounded(buffer_depth);
								let (wr_data_sender, wr_data_receiver) = parent.bounded(buffer_depth);
								let (ack_sender, ack_receiver) = parent.bounded(buffer_depth);
								let (rd_addr_sender, rd_addr_receiver) = parent.bounded(buffer_depth);
								let (rd_data_sender, rd_data_receiver) = parent.bounded(buffer_depth);

								let pmu_adapter_upstream = pmu_adapter_upstream::new(tile_receiver_vec, 1, wr_addr_sender, wr_data_sender, num_input as usize, pmu_counter_tmp, dummy);
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
								let tmp = pmu_counter_tmp * num_input;
								rd_addr_gen.set_run(move |time| {
									for idx in 0..tmp
									{
										ack_receiver.dequeue(time).unwrap();
										let curr_time = time.tick();
										rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
									}
								});
								parent.add_child(rd_addr_gen);
				
								let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tile_sender_vec, pmu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input as usize, pmu_counter_tmp, dummy);
								parent.add_child(pmu_adapter_downstream);










								let (sender, receiver) = parent.bounded(buffer_depth);
								let to_router_adapter = to_router_adapter::new(router_receiver_vec, pmu_sender_vec_tmp.len(), sender, num_input, dummy);
								parent.add_child(to_router_adapter);

								router_in_stream.push(receiver);

								let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
								router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;

								println!("PMU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
								if noc_topology == "mesh"
								{
									let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
									parent.add_child(router_mesh);
								} else if noc_topology == "sho_mesh"
								{

								} else if noc_topology == "sn_mesh"
								{

								} else
								{
									panic!("Wrong!");
								}



							} else if pmu_sender_vec_tmp[0] == no_connection
							{




								// let mut router_sender_vec = vec![];
								// let mut tile_receiver_vec = vec![];
								// let mut tile_sender_vec = vec![];
								// let mut tile_dst_vec = vec![];
								// for k in 0..pmu_receiver_vec_tmp.len()
								// {
								// 	let (sender, receiver) = parent.bounded(buffer_depth);
								// 	router_sender_vec.push(sender);
								// 	tile_receiver_vec.push(receiver);
								// }
								// tile_dst_vec.push(no_connection);

								// let (sender, receiver) = parent.bounded(buffer_depth);
								// tile_sender_vec.push(sender);
								
								





								// PMU setup
								// let (wr_addr_sender, wr_addr_receiver) = parent.bounded(buffer_depth);
								// let (wr_data_sender, wr_data_receiver) = parent.bounded(buffer_depth);
								// let (ack_sender, ack_receiver) = parent.bounded(buffer_depth);
								// let (rd_addr_sender, rd_addr_receiver) = parent.bounded(buffer_depth);
								// let (rd_data_sender, rd_data_receiver) = parent.bounded(buffer_depth);

								// let pmu_adapter_upstream = pmu_adapter_upstream::new(tile_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input as usize, pmu_counter_tmp, dummy);
								// parent.add_child(pmu_adapter_upstream);

								// let mut pmu: PMU<usize, usize, bool> = PMU::<usize, usize, bool>::new(
								// 	num_vec_per_pmu,
								// 	Behavior {
								// 		mod_address: false,
								// 		use_default_value: false,
								// 	},
								// );
								// pmu.add_writer(PMUWriteBundle {
								// 	addr: wr_addr_receiver,
								// 	data: wr_data_receiver,
								// 	ack: ack_sender,
								// });
								// pmu.add_reader(PMUReadBundle {
								// 	addr: rd_addr_receiver,
								// 	resp: rd_data_sender,
								// });
								// parent.add_child(pmu);
								
								// let mut rd_addr_gen = FunctionContext::new();
								// ack_receiver.attach_receiver(&rd_addr_gen);
								// rd_addr_sender.attach_sender(&rd_addr_gen);
								// let tmp = pmu_counter_tmp * num_input;
								// rd_addr_gen.set_run(move |time| {
								// 	for idx in 0..tmp
								// 	{
								// 		ack_receiver.dequeue(time).unwrap();
								// 		let curr_time = time.tick();
								// 		rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
								// 	}
								// });
								// parent.add_child(rd_addr_gen);
								
								// let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tile_sender_vec, 1, tile_dst_vec, num_input as usize, pmu_counter_tmp, dummy);
								// parent.add_child(pmu_adapter_downstream);









								let (sender, receiver) = parent.bounded(buffer_depth);
								router_out_stream.push(sender);

								let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
								router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;

								println!("PMU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
								if noc_topology == "mesh"
								{
									let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
									parent.add_child(router_mesh);
								} else if noc_topology == "sho_mesh"
								{

								} else if noc_topology == "sn_mesh"
								{

								} else
								{
									panic!("Wrong!");
								}



								// let from_router_adapter = from_router_adapter::new(receiver, router_sender_vec, pmu_receiver_vec_tmp.len(), num_input, dummy);
								// parent.add_child(from_router_adapter);


								let con = ConsumerContext::new(receiver);
								parent.add_child(con);




							} else if pmu_receiver_vec_tmp[0] == no_connection && pmu_sender_vec_tmp[0] == no_connection
							{
								panic!("Wrong!");


							} else
							{


								let mut tile_sender_vec = vec![];
								let mut tile_dst_vec = vec![];
								let mut router_receiver_vec = vec![];

								let mut router_sender_vec = vec![];
								let mut tile_receiver_vec = vec![];

								for k in 0..pmu_receiver_vec_tmp.len()
								{
									let (sender, receiver) = parent.bounded(buffer_depth);
									router_sender_vec.push(sender);
									tile_receiver_vec.push(receiver);
								}
								for k in 0..pmu_sender_vec_tmp.len()
								{
									let (sender, receiver) = parent.bounded(buffer_depth);
									tile_sender_vec.push(sender);
									router_receiver_vec.push(receiver);

									let mut connection_id = pmu_sender_vec_tmp[k];
									if connection_first_type[connection_id] == "pmu" && connection_first_x[connection_id] == x && connection_first_y[connection_id] == y
									{
										tile_dst_vec.push(connection_second_x[connection_id] * y_dim + connection_second_y[connection_id]);
									} else {
										panic!("Wrong!");
									}
								}

								


								// PMU setup
								let (wr_addr_sender, wr_addr_receiver) = parent.bounded(buffer_depth);
								let (wr_data_sender, wr_data_receiver) = parent.bounded(buffer_depth);
								let (ack_sender, ack_receiver) = parent.bounded(buffer_depth);
								let (rd_addr_sender, rd_addr_receiver) = parent.bounded(buffer_depth);
								let (rd_data_sender, rd_data_receiver) = parent.bounded(buffer_depth);

								let pmu_adapter_upstream = pmu_adapter_upstream::new(tile_receiver_vec, pmu_receiver_vec_tmp.len() as usize, wr_addr_sender, wr_data_sender, num_input as usize, pmu_counter_tmp, dummy);
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
								let tmp = pmu_counter_tmp * num_input;
								rd_addr_gen.set_run(move |time| {
									for idx in 0..tmp
									{
										ack_receiver.dequeue(time).unwrap();
										let curr_time = time.tick();
										rd_addr_sender.enqueue(time, ChannelElement{time: curr_time, data: usize::try_from(0).unwrap(),},).unwrap();
									}
								});
								parent.add_child(rd_addr_gen);
				
								let pmu_adapter_downstream = pmu_adapter_downstream::new(rd_data_receiver, tile_sender_vec, pmu_sender_vec_tmp.len() as usize, tile_dst_vec, num_input as usize, pmu_counter_tmp, dummy);
								parent.add_child(pmu_adapter_downstream);









								let (sender1, receiver1) = parent.bounded(buffer_depth);
								let (sender2, receiver2) = parent.bounded(buffer_depth);
								router_in_stream.push(receiver1);

								let tmp = used_link_map[&(x, y, "from_L".to_owned(), x, y, "from_L".to_owned())];
								router_in_dict.insert("L_in".to_owned(), (router_in_len, tmp));
								router_in_len += 1;

								router_out_stream.push(sender2);
								
								let tmp = used_link_map[&(x, y, "to_L".to_owned(), x, y, "to_L".to_owned())];
								router_out_dict.insert("L_out".to_owned(), (router_out_len, tmp));
								router_out_len += 1;






								let to_router_adapter = to_router_adapter::new(router_receiver_vec, pmu_sender_vec_tmp.len(), sender1, num_input, dummy);
								parent.add_child(to_router_adapter);

								println!("PMU: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
								if noc_topology == "mesh"
								{
									let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
									parent.add_child(router_mesh);
								} else if noc_topology == "sho_mesh"
								{

								} else if noc_topology == "sn_mesh"
								{

								} else
								{
									panic!("Wrong!");
								}




								let from_router_adapter = from_router_adapter::new(receiver2, router_sender_vec, pmu_receiver_vec_tmp.len(), num_input, dummy);
								parent.add_child(from_router_adapter);

							}
						}
					}
				}
			}




			// extra routers
			for ele in extra_routers
			{
				let x = ele.0;
				let y = ele.1;
				// router setup
				let mut router_in_stream = vec![];
				let mut router_in_dict: HashMap<String, (usize, usize)> = HashMap::new();
				let mut router_in_len = 0;
				
				let mut router_out_stream = vec![];
				let mut router_out_dict: HashMap<String, (usize, usize)> = HashMap::new();
				let mut router_out_len = 0;







				if noc_topology == "mesh"
				{
					// global links
					if receiver_map_noc_global.contains_key(&(x-1, y, "S".to_owned(), x, y, "N".to_owned()))
					{
						let N_in = receiver_map_noc_global.remove(&(x-1, y, "S".to_owned(), x, y, "N".to_owned())).unwrap();
						router_in_stream.push(N_in);
						
						let tmp = used_link_map[&(x-1, y, "S".to_owned(), x, y, "N".to_owned())];
						router_in_dict.insert("N_in".to_owned(), (router_in_len, tmp));
						router_in_len += 1;
					}
					
					if receiver_map_noc_global.contains_key(&(x+1, y, "N".to_owned(), x, y, "S".to_owned()))
					{
						let S_in = receiver_map_noc_global.remove(&(x+1, y, "N".to_owned(), x, y, "S".to_owned())).unwrap();
						router_in_stream.push(S_in);

						let tmp = used_link_map[&(x+1, y, "N".to_owned(), x, y, "S".to_owned())];
						router_in_dict.insert("S_in".to_owned(), (router_in_len, tmp));
						router_in_len += 1;
					}
					
					if receiver_map_noc_global.contains_key(&(x, y+1, "W".to_owned(), x, y, "E".to_owned()))
					{
						let E_in = receiver_map_noc_global.remove(&(x, y+1, "W".to_owned(), x, y, "E".to_owned())).unwrap();
						router_in_stream.push(E_in);

						let tmp = used_link_map[&(x, y+1, "W".to_owned(), x, y, "E".to_owned())];
						router_in_dict.insert("E_in".to_owned(), (router_in_len, tmp));
						router_in_len += 1;
					}
					
					if receiver_map_noc_global.contains_key(&(x, y-1, "E".to_owned(), x, y, "W".to_owned()))
					{
						let W_in = receiver_map_noc_global.remove(&(x, y-1, "E".to_owned(), x, y, "W".to_owned())).unwrap();
						router_in_stream.push(W_in);

						let tmp = used_link_map[&(x, y-1, "E".to_owned(), x, y, "W".to_owned())];
						router_in_dict.insert("W_in".to_owned(), (router_in_len, tmp));
						router_in_len += 1;
					}
					
					if sender_map_noc_global.contains_key(&(x, y, "N".to_owned(), x-1, y, "S".to_owned()))
					{
						let N_out = sender_map_noc_global.remove(&(x, y, "N".to_owned(), x-1, y, "S".to_owned())).unwrap();
						router_out_stream.push(N_out);

						let tmp = used_link_map[&(x, y, "N".to_owned(), x-1, y, "S".to_owned())];
						router_out_dict.insert("N_out".to_owned(), (router_out_len, tmp));
						router_out_len += 1;
					}
					
					if sender_map_noc_global.contains_key(&(x, y, "S".to_owned(), x+1, y, "N".to_owned()))
					{
						let S_out = sender_map_noc_global.remove(&(x, y, "S".to_owned(), x+1, y, "N".to_owned())).unwrap();
						router_out_stream.push(S_out);

						let tmp = used_link_map[&(x, y, "S".to_owned(), x+1, y, "N".to_owned())];
						router_out_dict.insert("S_out".to_owned(), (router_out_len, tmp));
						router_out_len += 1;
					}
					
					if sender_map_noc_global.contains_key(&(x, y, "E".to_owned(), x, y+1, "W".to_owned()))
					{
						let E_out = sender_map_noc_global.remove(&(x, y, "E".to_owned(), x, y+1, "W".to_owned())).unwrap();
						router_out_stream.push(E_out);

						let tmp = used_link_map[&(x, y, "E".to_owned(), x, y+1, "W".to_owned())];
						router_out_dict.insert("E_out".to_owned(), (router_out_len, tmp));
						router_out_len += 1;
					}
					
					if sender_map_noc_global.contains_key(&(x, y, "W".to_owned(), x, y-1, "E".to_owned()))
					{
						let W_out = sender_map_noc_global.remove(&(x, y, "W".to_owned(), x, y-1, "E".to_owned())).unwrap();	
						router_out_stream.push(W_out);

						let tmp = used_link_map[&(x, y, "W".to_owned(), x, y-1, "E".to_owned())];
						router_out_dict.insert("W_out".to_owned(), (router_out_len, tmp));
						router_out_len += 1;
					}
				} else if noc_topology == "sho_mesh"
				{

				} else if noc_topology == "sn_mesh"
				{

				} else
				{
					panic!("Wrong!");
				}






				



				println!("router: x{}, y{}, router_in_dict{:?}, router_out_dict{:?}", x, y, router_in_dict.keys(), router_out_dict.keys());
				if noc_topology == "mesh"
				{
					let router_mesh = router_mesh::new(router_in_stream, router_in_dict, router_in_len, router_out_stream, router_out_dict, router_out_len, x_dim, y_dim, x, y, num_input, num_vc, dummy);
					parent.add_child(router_mesh);
				} else if noc_topology == "sho_mesh"
				{

				} else if noc_topology == "sn_mesh"
				{

				} else
				{
					panic!("Wrong!");
				}


			}


			


			

		
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

			let dam_compute_time = executed.elapsed_cycles().unwrap();
			let dam_compute_time: f32 = dam_compute_time as f32 / num_input as f32 / freq as f32;
			println!("DFModel compute latency per tile {}", Compute_Latency[i]);
			println!("DFModel memory latency per tile {}", Memory_Latency[i]);
			println!("DFModel network latency per tile {}", Network_Latency[i]);
			println!("DFModel overall latency per tile {}", dfmodel_time[i]);
			println!("DAM compute latency per tile {}", dam_compute_time);
		}


		println!("\n\n\n\n\n\n\n");
	}

}
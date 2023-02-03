use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::time::Instant;

use nanorand::{Rng, WyRand};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use google_cloud_storage::client::Client;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::UploadObjectRequest;
use google_cloud_storage::http::Error;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::sign::SignedURLOptions;

use self::priority_queue::PriorityQueueItem;

mod priority_queue;

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
struct NodeID(u32);

// implement display options for printing during debug
impl fmt::Display for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Cost(u16);

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct LeavingTime(u32);

#[derive(Serialize, Deserialize, Clone, Copy)]
struct EdgeWalk {
    to: NodeID,
    cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
struct GraphWalk {
    edges_per_node: HashMap<usize, SmallVec<[EdgeWalk; 4]>>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct EdgePT {
    leavingTime: LeavingTime,
    cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
struct GraphPT {
    edges_per_node: HashMap<usize, SmallVec<[EdgePT; 4]>>,
}

fn main() {
    /// these are for dev only: understanding time to run different
    //assess_cost_of_casting();
    //test_vec_subset_speed();
    //demonstrate_mutable_q();

    //serialise_list("start_nodes");
    //serialise_list("init_travel_times");
    //serialise_GraphWalk();
    //serialise_GraphPT();
    //serialise_list_of_lists("node_values");
    //serialise_list_of_lists("travel_time_relationships");
    //serialise_hashmap_i8("subpurpose_purpose_lookup");
    let now = Instant::now();
    let start_nodes = read_serialised_vect32("start_nodes");
    let init_travel_times = read_serialised_vect32("init_travel_times");
    let graph_walk = read_GraphWalk();
    let graph_pt = read_GraphPT();
    let node_values = read_list_of_lists_vect32("node_values");
    let travel_time_relationships = read_list_of_lists_vect32("travel_time_relationships");
    let subpurpose_purpose_lookup = read_hashmap_i8("subpurpose_purpose_lookup");
    println!("Loading took {:?}", now.elapsed());

    let number_of_destination_categories = 5;
    let trip_start_seconds = 3600 * 8;

    // Loop through start nodes at random
    let mut rng = WyRand::new();
    let now = Instant::now();
    let mut score_store = Vec::new();
    let mut total_iters_counter = 0;
    for start_ix in 0..100 {
        //let start_ix = rng.generate_range(0..start_nodes.len());
        let start = NodeID((start_nodes[start_ix] as u32));
        let (total_iters, scores) = floodfill(
            &graph_walk,
            start,
            &node_values,
            &travel_time_relationships,
            &subpurpose_purpose_lookup,
            &graph_pt,
            trip_start_seconds,
        );

        // store
        total_iters_counter += total_iters;
        score_store.push(scores);
    }
    println!(
        "Calculating routes took {:?}\nReached {} nodes in total",
        now.elapsed(),
        total_iters_counter
    );
    println!("Score from last start node {:?}", score_store.pop());
}

fn floodfill(
    graph_walk: &GraphWalk,
    start: NodeID,
    node_values: &Vec<Vec<i32>>,
    travel_time_relationships: &Vec<Vec<i32>>,
    subpurpose_purpose_lookup: &HashMap<i8, i8>,
    graph_pt: &GraphPT,
    trip_start_seconds: i32,
) -> (i32, Vec<i32>) {
    let time_limit = Cost(3600);
    let subpurposes_count = node_values[0].len() as usize;
    let now = Instant::now();

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: Cost(0),
        value: start,
    });

    let mut nodes_visited = HashSet::new();
    let mut total_iters = 0;
    let mut pt_iters = 0;

    let mut scores: Vec<i32> = Vec::new();
    for i in 1..(subpurposes_count + 1) {
        scores.push(0);
    }

    while let Some(current) = queue.pop() {
        if nodes_visited.contains(&current.value) {
            continue;
        }
        if current.cost > time_limit {
            continue;
        }

        nodes_visited.insert(current.value);

        // if the node id is under 40m, then it will have an associated value
        if current.value.0 < 40_000_000 {
            // to do: node_values uses (what is probably) Expensive casting!
            // can the 'borrow' (&) be used to speed this up?
            // Can we change 'scores' inplace within the function to speed this up, perhaps
            // by making making 'scores' global (as we do in python)
            get_scores(
                &node_values[(current.value.0 as usize)],
                current.cost.0,
                travel_time_relationships,
                subpurpose_purpose_lookup,
                subpurposes_count,
                &mut scores,
            );
            /*
            let new_scores = get_scores(
                &node_values[(current.value.0 as usize)],
                current.cost.0,
                travel_time_relationships,
                subpurpose_purpose_lookup,
                subpurposes_count,
            );
            for i in 0..subpurposes_count {
                scores[i] += new_scores[i];
            }
            */
        }

        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        for edge in &graph_walk.edges_per_node[&(current.value.0 as usize)][1..] {
            queue.push(PriorityQueueItem {
                cost: Cost(current.cost.0 + edge.cost.0),
                value: edge.to,
            });
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        if graph_walk.edges_per_node[&(current.value.0 as usize)][0].cost == Cost(1) {
            let pt_connection = get_pt_connections(
                &graph_walk,
                &graph_pt, // alter this to be a vector of vectors
                current.cost.0,
                &queue,
                time_limit,
                trip_start_seconds,
                &current.value,
            );

            /// as get_pt_connections() doesn't push to queue inside the function, do it here (ideally change this to save cycles)
            // pt_connection.0.0 is seconds since start of simulation
            if pt_connection.0 .0 > 0 {
                queue.push(PriorityQueueItem {
                    cost: pt_connection.0,
                    value: pt_connection.1,
                });

                pt_iters += 1;
            }
        }

        total_iters += 1;
    }
    println!(
        "pt_iters: {}\ttotal_iters: {}\t{:?}",
        pt_iters,
        total_iters,
        now.elapsed()
    );

    return (total_iters, scores);
}

fn get_pt_connections(
    graph_walk: &GraphWalk,
    graph_pt: &GraphPT,
    time_so_far: u16,
    queue: &BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: &NodeID,
) -> (Cost, NodeID) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u32 = 0;
    let mut next_leaving_time = 0;
    for edge in &graph_pt.edges_per_node[&(current_node.0 as usize)][1..] {
        if time_of_arrival_current_node <= edge.cost.0 as u32 {
            next_leaving_time = edge.cost.0;
            journey_time = edge.leavingTime.0 as u32;
            found_next_service = 1;
            break;
        }
    }

    // export
    let mut output = (Cost(0 as u16), NodeID(0 as u32));
    if found_next_service == 1 {
        let wait_time_this_stop = next_leaving_time as u32 - time_of_arrival_current_node;
        let arrival_time_next_stop =
            time_so_far as u32 + wait_time_this_stop as u32 + journey_time as u32;

        if arrival_time_next_stop < time_limit.0 as u32 {
            //// prep output for queue. Notice this uses 'leavingTime' as first 'edge' for each node stores ID
            //// of next node: this is legacy from our matrix-based approach in python
            //// Todo: would be better to write to queue inplace to save shunting data around as much
            let destination_node = &graph_pt.edges_per_node[&(current_node.0 as usize)][0]
                .leavingTime
                .0;
            //println!("destination_node {}", destination_node);

            output = (
                Cost(arrival_time_next_stop as u16),
                NodeID(*destination_node as u32),
            );
        };
    }

    return output;
}

fn get_scores(
    values_this_node: &Vec<i32>,
    time_so_far: u16,
    travel_time_relationships: &Vec<Vec<i32>>,
    subpurpose_purpose_lookup: &HashMap<i8, i8>,
    subpurposes_count: usize,
    scores: &mut Vec<i32>,
) {
    for i in 0..subpurposes_count {
        let ix_purpose = subpurpose_purpose_lookup[&(i as i8)];
        scores[i] += values_this_node[i]
            * travel_time_relationships[ix_purpose as usize][time_so_far as usize];
    }
}

fn serialise_hashmap_i8(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: HashMap<i8, i8> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn read_hashmap_i8(filename: &str) -> HashMap<i8, i8> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: HashMap<i8, i8> = bincode::deserialize_from(file).unwrap();
    output
}

fn read_list_of_lists_vect32(filename: &str) -> Vec<Vec<i32>> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<Vec<i32>> = bincode::deserialize_from(file).unwrap();
    output
}

fn serialise_list_of_lists(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: Vec<Vec<i32>> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_GraphPT() {
    let contents = std::fs::read_to_string("data/p2_main_nodes.json").unwrap();

    // to do: check meaning of the '2' in [usize; 2]
    let input: HashMap<usize, Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    // make empty dict
    let mut graph = GraphPT {
        edges_per_node: HashMap::new(),
    };

    // populate dict
    for (from, input_edges) in input {
        let mut edges = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavingTime: LeavingTime(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph.edges_per_node.insert(from, edges);
    }

    let file = BufWriter::new(File::create("serialised_data/p2_main_nodes.bin").unwrap());
    bincode::serialize_into(file, &graph).unwrap();
}

fn read_GraphWalk() -> GraphWalk {
    let file = BufReader::new(File::open("serialised_data/p1_main_nodes.bin").unwrap());
    let output: GraphWalk = bincode::deserialize_from(file).unwrap();
    output
}

fn read_GraphPT() -> GraphPT {
    let file = BufReader::new(File::open("serialised_data/p2_main_nodes.bin").unwrap());
    let output: GraphPT = bincode::deserialize_from(file).unwrap();
    output
}

fn serialise_GraphWalk() {
    let contents = std::fs::read_to_string("data/p1_main_nodes.json").unwrap();

    // to do: check meaning of the '2' in [usize; 2]
    let input: HashMap<usize, Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    // make empty dict
    let mut graph = GraphWalk {
        edges_per_node: HashMap::new(),
    };

    // populate dict
    for (from, input_edges) in input {
        let mut edges = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph.edges_per_node.insert(from, edges);
    }

    let file = BufWriter::new(File::create("serialised_data/p1_main_nodes.bin").unwrap());
    bincode::serialize_into(file, &graph).unwrap();
}

fn serialise_list(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: Vec<i32> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn read_serialised_vect32(filename: &str) -> Vec<i32> {
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<i32> = bincode::deserialize_from(file).unwrap();
    output
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

/// this and push_to_q() are for reference only
fn demonstrate_mutable_q() {
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: Cost(0),
        value: NodeID(1),
    });
    push_to_q(&mut queue);
    push_to_q(&mut queue);
    while let Some(current) = queue.pop() {
        println!("{}, {}", current.value, current.cost);
    }
}

fn push_to_q(queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>) {
    queue.push(PriorityQueueItem {
        cost: Cost(1),
        value: NodeID(2),
    });
}

fn test_vec_subset_speed() {
    let mut VoV = Vec::new();

    //let mut VoV: <Vec<Vec<i32>>;
    for _ in 1..1000 {
        let mut scores: Vec<i32> = Vec::new();
        for i in 1..2000 {
            scores.push(0);
        }
        VoV.push(scores);
    }
    println!("VoV len: {:?}", VoV.len());
    println!("VoV inner len: {:?}", VoV[0].len());

    let now = Instant::now();
    let mut topps: i32 = 0;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            VoV[i][k];
            //iters += 1;
        }
    }
    println!("VoV took {:?}", now.elapsed());

    let now = Instant::now();
    let mut topps: i32 = 0;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            topps += VoV[i][k];
            //iters += 1;
        }
    }
    println!("VoV took {:?}\ttopps: {}", now.elapsed(), topps);

    let now = Instant::now();
    let mut topps: i32 = 0;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            topps += VoV[i][k];
            iters += 1;
        }
    }
    println!("VoV took {:?}\t with iters {}", now.elapsed(), iters);
    // all the above shows assigning to 'iters' is much more time intensive than subsetting:
    // dont bother with any other data structure
}

fn assess_cost_of_casting() {
    let mut VoV = Vec::new();

    //let mut VoV: <Vec<Vec<i32>>;
    for _ in 1..1000 {
        let mut scores: Vec<i32> = Vec::new();
        for i in 1..2000 {
            scores.push(0);
        }
        VoV.push(scores);
    }
    let now = Instant::now();
    let mut topps: i32 = 1;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            VoV[i][k] += topps;
            //iters += 1;
        }
    }
    println!("VoV without casting took {:?}", now.elapsed());

    let now = Instant::now();
    let mut topps: i16 = 1;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            VoV[i][k] += topps as i32;
            //iters += 1;
        }
    }
    println!("VoV WITH casting took {:?}, {}", now.elapsed(), VoV[5][5]);

    let now = Instant::now();
    let mut topps: i16 = 1;
    let mut iters: i32 = 0;
    for i in 0..999 {
        for k in 0..1999 {
            //VoV[i][k] += topps as i32;
            iters += topps as i32;
        }
    }
    println!("Topps WITH casting took {:?}, {}", now.elapsed(), iters);
}

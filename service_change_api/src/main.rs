use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use smallvec::SmallVec;
use std::time::Instant;
use typed_index_collections::TiVec;

use common::structs::{Cost, EdgePT, EdgeWalk, LeavingTime, Multiplier, NodeID, ServiceChangePayload, FloodfillOutputOriginDestinationPair};
use common::floodfill_public_transport_purpose_scores::floodfill_public_transport_purpose_scores;
use common::floodfill_funcs::::get_time_of_day_index;
use common::read_file_funcs::{
    read_files_parallel_inc_node_values,
    read_small_files_serial,
    deserialize_bincoded_file,
};

struct AppState {
    travel_time_relationships_all: Vec<Vec<Multiplier>>,
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[get("/get_node_id_count/")]
async fn get_node_id_count() -> String {
    let year: i32 = 2022;   //// TODO change this dynamically depending on when user hits this api... OR drop this from Rust api and store in py
    let graph_walk_len: i32 = deserialize_bincoded_file(&format!("graph_walk_len_{year}"));
    return serde_json::to_string(&graph_walk_len).unwrap();
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<ServiceChangePayload>) -> String {

    println!(
        "Floodfill request received with year {input.year}\ninput.new_build_additions.len(): {}",
        input.new_build_additions.len()
    );

    let (mut node_values_2d, mut graph_walk, mut graph_routes) =
        read_files_parallel_inc_node_values(input.year);
    
    let graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let graph_routes: TiVec<NodeID, NodeRoute> = TiVec::from(graph_routes);
    let node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);

    let len_graph_walk = graph_walk.len();
    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);
    
    // TODO: check meaning of graph_walk_additions
    for input_edges in input.graph_walk_additions.iter() {
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk.push(NodeWalk{
            edges: edges,
            has_pt: 0
        });
    }

    // TODO check this
    for input_edges in input.graph_routes_additions.iter() {
        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavetime: LeavingTime(array[0] as u32),
                cost: Cost(array[1] as u16),
            });
        }
        graph_routes.push(edges);
    }
    
    assert!(graph_walk.len() == len_graph_walk + input.new_nodes_count);

    // TODO: check updates to graph walk reflect the new routes
    for i in 0..input.graph_walk_updates_keys.len() {
        let node = input.graph_walk_updates_keys[i];

        // TODO Just modify in-place
        let mut edges: SmallVec<[EdgeWalk; 4]> = graph_walk[node].clone();
        for array in &input.graph_walk_updates_additions[i] {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk[node] = edges;
    }

    // Altering node_values to reflect changes in graph
    // TODO alter to include new walk links and routes
    for _i in 0..input.graph_walk_additions.len() {
        let empty_vec: Vec<[i32; 2]> = Vec::new();
        node_values_2d.push(empty_vec);
    }

    for new_build in &input.new_build_additions {
        let value_to_add = new_build[0];
        let index_of_nearest_node = new_build[1];
        let subpurpose_ix = new_build[2];
        
        // add node value to current score if one can be found for this node for the new build's subpurpose
        let mut loop_ix = 0;
        let mut found_existing_subpurpose = false;
        let values_vec_this_node = node_values_2d[index_of_nearest_node as usize].to_vec();
        for subpurpose_score_pair in values_vec_this_node.iter() {
            let subpurpose_ix_existing = subpurpose_score_pair[0];
            if subpurpose_ix == subpurpose_ix_existing {
                node_values_2d[index_of_nearest_node as usize][loop_ix][1] += value_to_add;
                found_existing_subpurpose = true;
            }
            loop_ix += 1;
        }
        // append to node_values_2d if no value for that node's subpurpose
        if !found_existing_subpurpose {
            let subpurpose_value_to_add: [i32; 2] = [subpurpose_ix, value_to_add];
            node_values_2d[index_of_nearest_node as usize].push(subpurpose_value_to_add);
        }
    }
    
    let now = Instant::now();
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    // TODO update inputs to this
    let results: Vec<FloodfillOutputOriginDestinationPair> = indices
        .par_iter()
        .map(|i| {
            get_all_scores_and_time_to_target_destinations(
                &travel_times[*i],
                &node_values_2d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                &input.target_destinations,
            )
        })
        .collect();
    println!("Getting destinations and scores took {:?}", now.elapsed());
        
    serde_json::to_string(&results).unwrap()
}


fn get_travel_times_multicore(
    graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_routes: &Vec<SmallVec<[EdgePT; 4]>>,
    input: &web::Json<UserInputJSON>
) -> Vec<(u32, Vec<u32>, Vec<u16>)> {
        
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    return indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &graph_walk,
                &graph_routes,
                NodeID(*&input.start_nodes_user_input[*i] as u32),
                *&input.trip_start_seconds,
                Cost(*&input.init_travel_times_user_input[*i] as u16),
            )
        })
        .collect(); 
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
        
    let (
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        _subpurpose_purpose_lookup,
    ) = read_small_files_serial();

    let travel_time_relationships_all = vec![
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
    ];
    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
    });
    HttpServer::new(move || {
        App::new()
            // This clone is of an Arc from actix. AppState is immutable, and only one copy exists
            // (except for when we clone some pieces of it to make mutations scoped to a single
            // request.)
            .app_data(app_state.clone())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 500)) // allow POST'd JSON payloads up to 500mb
            .service(index)
            .service(get_node_id_count)
            .service(floodfill_pt)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}
use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use smallvec::SmallVec;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Arc;

use crate::shared::{Cost, EdgePT, EdgeWalk, NodeID, UserInputJSON};
use floodfill::{get_travel_times, get_all_scores_links_and_key_destinations};
use get_time_of_day_index::get_time_of_day_index;
use crate::read_files::{
    read_files_parallel_excluding_node_values,
    read_small_files_serial,
    deserialize_bincoded_file,
    create_graph_walk_len,
    read_sparse_node_values_2d_serial,
};
use make_and_serialise_nodes_within_120s::make_and_serialise_nodes_within_120s;

mod floodfill;
mod get_time_of_day_index;
mod priority_queue;
mod read_files;
mod serialise_files;
mod shared;
mod make_and_serialise_nodes_within_120s;



struct AppState {
    travel_time_relationships_all: Vec<Vec<i32>>,
    subpurpose_purpose_lookup: [i8; 32],
    arc_nodes_to_neighbouring_nodes: Arc<Vec<Vec<u32>>>,
    graph_walk: Arc<Vec<SmallVec<[EdgeWalk; 4]>>>,
    graph_pt: Arc<Vec<SmallVec<[EdgePT; 4]>>>,
    node_values_padding_row_count: u32, 
    node_values_2d: Arc<Vec<Vec<[i32;2]>>>,
}



fn get_travel_times_multicore(
    graph_walk: &Arc<Vec<SmallVec<[EdgeWalk; 4]>>>,
    graph_pt: &Arc<Vec<SmallVec<[EdgePT; 4]>>>,
    input: &web::Json<UserInputJSON>
) -> Vec<(u32, Vec<u32>, Vec<u16>, Vec<Vec<u32>>, u16)> {
        
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    return indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &graph_walk,
                &graph_pt,
                NodeID(*&input.start_nodes_user_input[*i] as u32),
                *&input.trip_start_seconds,
                Cost(*&input.init_travel_times_user_input[*i] as u16),
                false,
                3600,
            )
        })
        .collect(); 
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
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {

    println!("Floodfill request received");
    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);
    let count_original_nodes = data.graph_walk.len() as u32;
    
    println!(
        "Started running floodfill and node values files read\ttime_of_day_ix: {}\tNodes count: {}",
        time_of_day_ix,
        input.start_nodes_user_input.len()
    );
    
    let now = Instant::now();
    
    let floodfill_outputs_tuple = get_travel_times_multicore(
        &data.graph_walk,
        &data.graph_pt,
        &input,
    );
    
    /*
    // The old version
    let (node_values_2d, floodfill_outputs_tuple) = parallel_node_values_read_and_floodfill(
        &data.graph_walk,
        &data.graph_pt,
        &input,
    );
    */
    
    println!("Node values read in and floodfill in parallel {:?}", now.elapsed());
    
    let now = Instant::now();
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    // [HashMap<f64, f64>; 5]
    let results: Vec<(i32, u32, [f64; 5], HashMap<u32, [f64; 5]>, HashMap<u32, [u32; 2]>, [HashMap<u32, Vec<u32>>; 5], u16)> = indices
        .par_iter()
        .map(|i| {
            get_all_scores_links_and_key_destinations(
                &floodfill_outputs_tuple[*i],
                &data.node_values_2d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                count_original_nodes,
                data.node_values_padding_row_count,
                &data.arc_nodes_to_neighbouring_nodes, 
            )
        })
        .collect();
    println!("Getting destinations, scores, link importances and clusters took {:?}", now.elapsed());
    
    serde_json::to_string(&results).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let year: i32 = 2022;
    
    // make this true on initial run; false otherwise
    if false {
        serialise_files::serialise_files(year);
        serialise_files::serialise_sparse_node_values_2d(year);
        create_graph_walk_len(year); 
    }
    
    // comment this out to not make the lookup of nodes which are near other nodes
    // this is big preprocessing stage (~90mins with 8cores)
    // make_and_serialise_nodes_within_120s(year);
    
    let (
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    ) = read_small_files_serial();

    let travel_time_relationships_all = vec![
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
    ];
    
    let (graph_walk, graph_pt, node_values_padding_row_count) =
        read_files_parallel_excluding_node_values(2022);
    
    let node_values_2d = read_sparse_node_values_2d_serial(2022);
    
    let graph_walk = Arc::new(graph_walk);
    let graph_pt = Arc::new(graph_pt);
    let node_values_2d = Arc::new(node_values_2d);
    
    let nodes_to_neighbouring_nodes: Vec<Vec<u32>> = deserialize_bincoded_file("nodes_to_neighbouring_nodes");
    let arc_nodes_to_neighbouring_nodes: Arc<Vec<Vec<u32>>> = Arc::new(nodes_to_neighbouring_nodes);
    
    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        subpurpose_purpose_lookup,
        arc_nodes_to_neighbouring_nodes,
        graph_walk,
        graph_pt,
        node_values_padding_row_count,
        node_values_2d,
    });
    HttpServer::new(move || {
        App::new()
            // This clone is of an Arc from actix. AppState is immutable, and only one copy exists
            // (except for when we clone some pieces of it to make mutations scoped to a single
            // request.)
            .app_data(app_state.clone())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 50)) // allow POST'd JSON payloads up to 50mb
            .service(index)
            .service(get_node_id_count)
            .service(floodfill_pt)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}

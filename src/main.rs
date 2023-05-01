use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;
use typed_index_collections::TiVec;

use crate::read_files::{
    deserialize_bincoded_file, read_files_parallel_excluding_node_values,
    read_rust_node_longlat_lookup_serial, read_small_files_serial,
    read_sparse_node_values_2d_serial,
};
use crate::shared::{
    Cost, FinalOutput, FloodfillOutput, Multiplier, NodeID, NodePT, NodeWalk, SubpurposeScore,
    UserInputJSON,
};
use floodfill::{get_all_scores_links_and_key_destinations, get_travel_times};
use get_time_of_day_index::get_time_of_day_index;

mod floodfill;
mod get_time_of_day_index;
mod make_and_serialise_nodes_within_120s;
mod priority_queue;
mod read_files;
mod serialise_files;
mod shared;

struct AppState {
    travel_time_relationships_all: Vec<Vec<Multiplier>>,
    subpurpose_purpose_lookup: [usize; 32],
    nodes_to_neighbouring_nodes: TiVec<NodeID, Vec<NodeID>>,
    graph_walk: TiVec<NodeID, NodeWalk>,
    graph_pt: TiVec<NodeID, NodePT>,
    node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>>,
    rust_node_longlat_lookup: TiVec<NodeID, [f64; 2]>,
    route_info: TiVec<NodeID, HashMap<String, String>>, // TiVec<NodeID, String>,
}

fn get_travel_times_multicore(
    graph_walk: &TiVec<NodeID, NodeWalk>,
    graph_pt: &TiVec<NodeID, NodePT>,
    input: &web::Json<UserInputJSON>,
) -> Vec<FloodfillOutput> {
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();

    return indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &graph_walk,
                &graph_pt,
                *&input.start_nodes_user_input[*i],
                *&input.trip_start_seconds,
                *&input.init_travel_times_user_input[*i],
                false,
                Cost(3600),
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
    let year: i32 = 2022; //// TODO change this dynamically depending on when user hits this api... OR drop this from Rust api and store in py
    let graph_walk_len: i32 = deserialize_bincoded_file(&format!("graph_walk_len_{year}"));
    return serde_json::to_string(&graph_walk_len).unwrap();
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<UserInputJSON>) -> String {
    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);

    println!(
        "Started running floodfill and node values files read\ttime_of_day_ix: {}\tNodes count: {}",
        time_of_day_ix,
        input.start_nodes_user_input.len()
    );

    let now = Instant::now();

    let floodfill_outputs = get_travel_times_multicore(&data.graph_walk, &data.graph_pt, &input);

    println!("Floodfill in {:?}", now.elapsed());

    let now = Instant::now();
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();

    let results: Vec<FinalOutput> = indices
        .par_iter()
        .map(|i| {
            get_all_scores_links_and_key_destinations(
                &floodfill_outputs[*i],
                &data.node_values_2d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
                &data.nodes_to_neighbouring_nodes,
                &data.rust_node_longlat_lookup,
                &data.route_info,
            )
        })
        .collect();
    println!(
        "Getting destinations, scores, link importances and clusters took {:?}",
        now.elapsed()
    );

    serde_json::to_string(&results).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let year: i32 = 2022;

    // make this true on initial run; false otherwise
    if false {
        serialise_files::serialise_files(year);
        serialise_files::serialise_sparse_node_values_2d(year);
        serialise_files::serialise_rust_node_longlat_lookup(year);
    }

    // comment this out to not make the lookup of nodes which are near other nodes
    // this is big preprocessing stage (~90mins with 8cores)
    if false {
        let (graph_walk, graph_pt) = read_files_parallel_excluding_node_values(year);
        make_and_serialise_nodes_within_120s::make_and_serialise_nodes_within_120s(
            graph_walk, graph_pt,
        );
    }

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

    let route_info: Vec<HashMap<String, String>> =
        deserialize_bincoded_file(&format!("route_info_{year}"));
    let (graph_walk, graph_pt) = read_files_parallel_excluding_node_values(year);
    let node_values_2d = read_sparse_node_values_2d_serial(year);
    let rust_node_longlat_lookup = read_rust_node_longlat_lookup_serial();
    let nodes_to_neighbouring_nodes: Vec<Vec<NodeID>> =
        deserialize_bincoded_file("nodes_to_neighbouring_nodes");

    let now = Instant::now();
    let graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let graph_pt: TiVec<NodeID, NodePT> = TiVec::from(graph_pt);
    let node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);
    let rust_node_longlat_lookup: TiVec<NodeID, [f64; 2]> = TiVec::from(rust_node_longlat_lookup);
    let nodes_to_neighbouring_nodes: TiVec<NodeID, Vec<NodeID>> =
        TiVec::from(nodes_to_neighbouring_nodes);
    let route_info: TiVec<NodeID, HashMap<String, String>> = TiVec::from(route_info);
    println!("Conversion to TiVec's took {:?} seconds", now.elapsed());

    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        subpurpose_purpose_lookup,
        nodes_to_neighbouring_nodes,
        graph_walk,
        graph_pt,
        node_values_2d,
        rust_node_longlat_lookup,
        route_info,
    });
    println!("Starting server");
    // The 500MB warning is wrong, the decorator on line below silences it
    #[allow(deprecated)]
    HttpServer::new(move || {
        App::new()
            // TODO Fix before deploying for real!
            .wrap(actix_cors::Cors::permissive())
            .app_data(app_state.clone())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 500)) // allow POST'd JSON payloads up to 500mb to cover all eventualities
            .service(index)
            .service(get_node_id_count)
            .service(floodfill_pt)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}

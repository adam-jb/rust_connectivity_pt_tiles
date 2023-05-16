use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use typed_index_collections::TiVec;

use common::read_file_funcs::{
    deserialize_bincoded_file, read_files_parallel_excluding_node_values,
    read_rust_node_longlat_lookup_serial, read_small_files_serial,
    read_sparse_node_values_2d_serial,
};
use common::structs::{
    Cost, Multiplier, NodeID, NodeRoute, NodeWalk, Score, SubpurposeScore, UserInputJSON,
};
use common::floodfill_public_transport_no_scores::floodfill_public_transport_no_scores;
use common::floodfill_funcs::get_time_of_day_index;
use get_all_scores_links_and_key_destinations::get_all_scores_links_and_key_destinations;

mod get_all_scores_links_and_key_destinations;

use std::env;

struct AppState {
    travel_time_relationships_all: Vec<Vec<Multiplier>>,
    nodes_to_neighbouring_nodes: TiVec<NodeID, Vec<NodeID>>,
    graph_walk: TiVec<NodeID, NodeWalk>,
    graph_pt: TiVec<NodeID, NodeRoute>,
    node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>>,
    rust_node_longlat_lookup: TiVec<NodeID, [f64; 2]>,
    route_info: TiVec<NodeID, HashMap<String, String>>,
    mutex_sparse_node_values_contributed: Mutex<TiVec<NodeID, [Score; 5]>>,
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

    let now = Instant::now();
    // Only looks at first input if the request has >1 start nodes
    let floodfill_output = floodfill_public_transport_no_scores(
        &data.graph_walk,
        &data.graph_pt,
        *&input.start_nodes_user_input[0],
        *&input.trip_start_seconds,
        *&input.init_travel_times_user_input[0],
        false,
        Cost(3600),
    );
    println!("Floodfill in {:?}", now.elapsed());
    
    let now = Instant::now();
    let results = get_all_scores_links_and_key_destinations(
        &floodfill_output,
        &data.node_values_2d,
        &data.travel_time_relationships_all[time_of_day_ix],
        &data.nodes_to_neighbouring_nodes,
        &data.rust_node_longlat_lookup,
        &data.route_info,
        &data.mutex_sparse_node_values_contributed,
    );

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
    
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    

    let year: i32 = 2022;
    let seconds_travel_for_destination_clustering = 120;
    
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

    let route_info: Vec<HashMap<String, String>> =
        deserialize_bincoded_file(&format!("route_info_{year}"));
    let (graph_walk, graph_pt) = read_files_parallel_excluding_node_values(year);
    let node_values_2d = read_sparse_node_values_2d_serial(year);
    let rust_node_longlat_lookup = read_rust_node_longlat_lookup_serial();
    let nodes_to_neighbouring_nodes: Vec<Vec<NodeID>> = deserialize_bincoded_file(
        format!(
            "nodes_to_neighbouring_nodes_{}",
            seconds_travel_for_destination_clustering
        )
        .as_str(),
    );

    let now = Instant::now();
    let graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let graph_pt: TiVec<NodeID, NodeRoute> = TiVec::from(graph_pt);
    let node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);
    let rust_node_longlat_lookup: TiVec<NodeID, [f64; 2]> = TiVec::from(rust_node_longlat_lookup);
    let nodes_to_neighbouring_nodes: TiVec<NodeID, Vec<NodeID>> =
        TiVec::from(nodes_to_neighbouring_nodes);
    let route_info: TiVec<NodeID, HashMap<String, String>> = TiVec::from(route_info);
    println!("Conversion to TiVec's took {:?} seconds", now.elapsed());

    // create mutex of empty values that nodes contribute. Do this now to save 0.4seconds initialising whenever the API is called. It is reset after each API call
    let now = Instant::now();
    let sparse_node_values_contributed: Vec<[Score; 5]> = (0..graph_walk.len())
        .into_par_iter()
        .map(|_| [Score::default(); 5])
        .collect();
    let non_mutex_sparse_node_values_contributed: TiVec<NodeID, [Score; 5]> =
        TiVec::from(sparse_node_values_contributed);
    let mutex_sparse_node_values_contributed = Mutex::new(non_mutex_sparse_node_values_contributed);
    println!("Making sparse node values took {:?}", now.elapsed());

    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        nodes_to_neighbouring_nodes,
        graph_walk,
        graph_pt,
        node_values_2d,
        rust_node_longlat_lookup,
        route_info,
        mutex_sparse_node_values_contributed,
    });
    println!("Starting server");
    // The 500MB warning is wrong, the decorator on line below silences it
    #[allow(deprecated)]
    HttpServer::new(move || {
        App::new()
            // TODO only allow certain CORS origins before deploying for real!
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

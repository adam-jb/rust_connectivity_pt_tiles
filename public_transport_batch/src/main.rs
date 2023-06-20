use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use std::time::Instant;
use typed_index_collections::TiVec;

use common::read_file_funcs::{
    read_files_parallel_excluding_node_values,
    read_small_files_serial,
    read_sparse_node_values_2d_serial,
    read_stop_rail_statuses,
};
use common::structs::{Cost, NodeID, Multiplier, NodeWalk, NodeRoute, SubpurposeScore, FloodfillOutputOriginDestinationPair, OriginDestinationUserInputJSON};
use common::floodfill_public_transport_purpose_scores::floodfill_public_transport_purpose_scores;
use common::floodfill_funcs::get_time_of_day_index;

struct AppState {
    travel_time_relationships_all: Vec<Vec<Multiplier>>,
    graph_walk: TiVec<NodeID, NodeWalk>,
    graph_routes: TiVec<NodeID, NodeRoute>,
    node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>>,
    stop_rail_statuses: TiVec<NodeID, bool>,
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<OriginDestinationUserInputJSON>) -> String {
    
    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);

    println!(
        "Started running floodfill and node values files read\ttime_of_day_ix: {}\tNodes count: {}",
        time_of_day_ix,
        input.start_nodes.len()
    );

    let now = Instant::now();
    let indices = (0..input.start_nodes.len()).collect::<Vec<_>>();
    
    let results: Vec<FloodfillOutputOriginDestinationPair> = indices
        .par_iter()
        .map(|i| {
            floodfill_public_transport_purpose_scores(
                &data.graph_walk,
                &data.graph_routes,
                *&input.start_nodes[*i],
                *&input.trip_start_seconds,
                *&input.init_travel_times[*i],
                false,
                Cost(3600),
                &data.node_values_2d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &input.destination_nodes,
                &data.stop_rail_statuses,
            )
        })
        .collect();
    
    println!("Floodfill in {:?}", now.elapsed());
    println!("results len {}", results.len());
    serde_json::to_string(&results).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let year: i32 = 2022;

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

    let (graph_walk, graph_routes) = read_files_parallel_excluding_node_values(year);
    let node_values_2d = read_sparse_node_values_2d_serial(year);
    
    let graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let graph_routes: TiVec<NodeID, NodeRoute> = TiVec::from(graph_routes);
    let node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);
    
    let stop_rail_statuses_input = read_stop_rail_statuses(year);
    let stop_rail_statuses: TiVec<NodeID, bool> = TiVec::from(stop_rail_statuses_input);

    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        graph_walk,
        graph_routes,
        node_values_2d,
        stop_rail_statuses,
    });
    println!("Starting server");
    // The 500MB warning is wrong, so we 'allow deprecated' to hide it
    #[allow(deprecated)]
    HttpServer::new(move || {
        App::new()
            // TODO Fix before deploying for real!
            .wrap(actix_cors::Cors::permissive())
            .app_data(app_state.clone())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 500)) // allow POST'd JSON payloads up to 500mb
            .service(index)
            .service(floodfill_pt)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}
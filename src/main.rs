use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use smallvec::SmallVec;
use std::time::Instant;

use crate::read_files::{
    read_files_parallel_excluding_node_values,
    read_small_files_serial,
    read_sparse_node_values_2d_serial,
};
use crate::shared::{Cost, NodeID, EdgePT, EdgeWalk, FloodfillOutput, UserInputJSON};
use floodfill::get_travel_times_and_scores;
use get_time_of_day_index::get_time_of_day_index;

mod floodfill;
mod get_time_of_day_index;
mod priority_queue;
mod read_files;
mod serialise_files;
mod shared;

struct AppState {
    travel_time_relationships_all: Vec<Vec<i32>>,
    subpurpose_purpose_lookup: [i8; 32],
    graph_walk: Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: Vec<SmallVec<[EdgePT; 4]>>,
    node_values_2d: Vec<Vec<[i32; 2]>>,
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
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
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    let results: Vec<FloodfillOutput> = indices
        .par_iter()
        .map(|i| {
            get_travel_times_and_scores(
                &data.graph_walk,
                &data.graph_pt,
                NodeID(*&input.start_nodes_user_input[*i] as u32),
                *&input.trip_start_seconds,
                Cost(*&input.init_travel_times_user_input[*i] as u16),
                3600,
                &data.node_values_2d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &data.subpurpose_purpose_lookup,
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

    // make this true on initial run; false otherwise
    if false {
        serialise_files::serialise_files(year);
        serialise_files::serialise_sparse_node_values_2d(year);
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

    let (graph_walk, graph_pt) = read_files_parallel_excluding_node_values(2022);
    let node_values_2d = read_sparse_node_values_2d_serial(2022);

    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        subpurpose_purpose_lookup,
        graph_walk,
        graph_pt,
        node_values_2d,
    });
    println!("Starting server");
    // The 50MB warning is wrong
    #[allow(deprecated)]
    HttpServer::new(move || {
        App::new()
            // TODO Fix before deploying for real!
            .wrap(actix_cors::Cors::permissive())
            .app_data(app_state.clone())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 50)) // allow POST'd JSON payloads up to 50mb
            .service(index)
            .service(floodfill_pt)
    })
    .bind(("0.0.0.0", 6000))?
    .run()
    .await
}

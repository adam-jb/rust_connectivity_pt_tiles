use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use std::time::Instant;
use typed_index_collections::TiVec;

use common::structs::{Cost, NodeID, SubpurposeScore, NodeWalkCyclingCar, WalkCyclingCarUserInputJSON, FloodfillWalkCyclingCarOutput};
use common::floodfill_walk_cycling_car::{floodfill_walk_cycling_car};
use common::read_file_funcs::read_files_serial_walk_cycling_car;
//use common::floodfill_funcs::get_time_of_day_index;

#[get("/")]
async fn index() -> String {
    println!("Ping received");
    format!("App is listening")
}

#[post("/floodfill_endpoint/")]
async fn floodfill_endpoint(input: web::Json<WalkCyclingCarUserInputJSON>) -> String {
        
    // Read in files at endpoint as we don't know which mode the user will request
    let (travel_time_relationships, node_values_2d, graph_walk) =
        read_files_serial_walk_cycling_car(&input.mode);
    
    let graph_walk: TiVec<NodeID, NodeWalkCyclingCar> = TiVec::from(graph_walk);
    let node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);
    
    // Extract costs of turning
    let time_costs_turn: [Cost; 4];
    if input.mode == "cycling" {
        time_costs_turn = [Cost(0), Cost(15), Cost(15), Cost(5)];
    } else {
        time_costs_turn = [Cost(0), Cost(0), Cost(0), Cost(0)];
    }
    
    let now = Instant::now();
    
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    let results: Vec<FloodfillWalkCyclingCarOutput> = indices
        .par_iter()
        .map(|i| {
            floodfill_walk_cycling_car(
                &travel_time_relationships,
                &node_values_2d,
                &graph_walk,
                &time_costs_turn,
                *&input.start_nodes_user_input[*i],
                *&input.init_travel_times_user_input[*i],
                &input.destination_nodes,
                Cost(3600),
            )
        })
        .collect(); 

    println!("Getting destinations and scores took {:?}", now.elapsed());
    serde_json::to_string(&results).unwrap()
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    // The 500MB warning is wrong, the decorator on line below silences it
    #[allow(deprecated)]
    HttpServer::new(move || {
        App::new()
            .data(web::JsonConfig::default().limit(1024 * 1024 * 500)) // allow POST'd JSON payloads up to 500mb
            .service(index)
            .service(floodfill_endpoint)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}

use actix_web::{get, post, web, App, HttpServer};
use rayon::prelude::*;
use std::time::Instant;
use typed_index_collections::TiVec;

use common::structs::{Cost, NodeID, SubpurposeScore, NodeWalkCyclingCar, WalkCyclingCarUserInputJSON, FloodfillOutputOriginDestinationPair, SecondsPastMidnight};
use common::floodfill_walk_cycling_car::{floodfill_walk_cycling_car};
use common::read_file_funcs::read_files_serial_walk_cycling_car;
use common::floodfill_funcs::get_time_of_day_index;

#[get("/")]
async fn index() -> String {
    println!("Ping received");
    format!("App is listening")
}

#[post("/floodfill_endpoint/")]
async fn floodfill_endpoint(input: web::Json<WalkCyclingCarUserInputJSON>) -> String {
    
    let time_of_day_index = get_time_of_day_index(input.trip_start_seconds);
    
    let mut start_time_group: usize = 7;
    if time_of_day_index == 1 {
        start_time_group = 10;
    }
    if time_of_day_index == 2 {
        start_time_group = 16;
    }
    if time_of_day_index == 3 {
        start_time_group = 19;
    }
    
    if input.trip_start_seconds == SecondsPastMidnight(1) && input.mode == "car" {
        start_time_group = 1;
    }
    
    println!("start_time_group {} for trip_start_seconds {}", start_time_group, input.trip_start_seconds.0);
    
    // Read in files at endpoint rather than in advance as we don't know which mode the user will request
    let (travel_time_relationships, node_values_2d, graph) =
        read_files_serial_walk_cycling_car(&input.mode, start_time_group);
    
    let graph: TiVec<NodeID, NodeWalkCyclingCar> = TiVec::from(graph);
    let mut node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);
    
    // If any destinations are to be removed prior to running floodfill
    for build_to_remove in input.builds_to_remove.iter() {
        let build_to_remove_subpurpose = build_to_remove[1];
        let node_id = NodeID(build_to_remove[0]);
        
        let mut index_to_remove = 9999;

        for (i, subpurpose_value) in node_values_2d[node_id].iter().enumerate() {
            if subpurpose_value.subpurpose_ix == build_to_remove_subpurpose {
                index_to_remove = i;
                println!("Destination to be dropped for i {} and subpurpose ix {} and subpurpose_score {}", i, subpurpose_value.subpurpose_ix, subpurpose_value.subpurpose_score.0);
            }
        }

        if index_to_remove != 9999 {
            node_values_2d[node_id].remove(index_to_remove);
        }

    }
    
    // Extract costs of turning, in order of: straight, right turn, u-turn, left turn
    let time_costs_turn: [Cost; 4];
    if input.mode == "cycling" {
        time_costs_turn = [Cost(0), Cost(15), Cost(15), Cost(5)];
        
    // If using the distance driving graph (ie, each edge is in terms of distance in metres, divided by 20, rather than time to cross the edge in seconds)
    } else if input.mode == "car" && input.trip_start_seconds == SecondsPastMidnight(1) {
        time_costs_turn = [Cost(0), Cost(0), Cost(0), Cost(0)];
    
    } else if input.mode == "car" {
        time_costs_turn = [Cost(0), Cost(15), Cost(17), Cost(9)];
        
    // walking turn costs
    } else {
        time_costs_turn = [Cost(0), Cost(0), Cost(0), Cost(0)];
    }
    
    let now = Instant::now();
    
    let indices = (0..input.start_nodes_user_input.len()).collect::<Vec<_>>();
    
    let results: Vec<FloodfillOutputOriginDestinationPair> = indices
        .par_iter()
        .map(|i| {
            floodfill_walk_cycling_car(
                &travel_time_relationships,
                &node_values_2d,
                &graph,
                &time_costs_turn,
                *&input.start_nodes_user_input[*i],
                *&input.init_travel_times_user_input[*i],
                &input.destination_nodes,
                Cost(3600),
                &input.mode,
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

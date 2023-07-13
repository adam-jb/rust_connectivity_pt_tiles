use actix_web::{get, post, web, App, HttpServer};
use log::{info, LevelFilter};
use rayon::prelude::*;
use smallvec::SmallVec;
use std::time::Instant;
use typed_index_collections::TiVec;

use common::floodfill_funcs::get_time_of_day_index;
use common::floodfill_public_transport_purpose_scores::floodfill_public_transport_purpose_scores;
use common::read_file_funcs::{
    deserialize_bincoded_file, read_files_extra_parallel_inc_node_values, read_small_files_serial,
    read_stop_rail_statuses, read_small_medium_large_subpurpose_destinations, 
};
use common::structs::{
    Cost, EdgeRoute, EdgeWalk, FloodfillOutputOriginDestinationPair, Multiplier, NodeID, NodeRoute,
    NodeWalk, Score, SecondsPastMidnight, ServiceChangePayload, SubpurposeScore, SubpurposeSmallMediumLargeCount, 
};

struct AppState {
    travel_time_relationships_all: Vec<Vec<Multiplier>>,
    small_medium_large_subpurpose_destinations: TiVec<NodeID, Vec<SubpurposeSmallMediumLargeCount>>,
}

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[get("/get_node_id_count/")]
async fn get_node_id_count() -> String {
    let year: i32 = 2022; //// TODO change this dynamically depending on when user hits this api... OR drop this from Rust api and store in py
    let graph_walk_len: i32 = deserialize_bincoded_file(&format!("graph_pt_walk_len_{year}"));
    return serde_json::to_string(&graph_walk_len).unwrap();
}

#[post("/floodfill_pt/")]
async fn floodfill_pt(data: web::Data<AppState>, input: web::Json<ServiceChangePayload>) -> String {
    info!("Received payload:\n{:?}", input);

    println!(
        "Floodfill request received with year {}\ninput.new_build_additions.len(): {}",
        input.year,
        input.new_build_additions.len()
    );
    
    let mut count_destinations_at_intervals = false;
    if *&input.count_destinations_at_intervals == 1 {
        count_destinations_at_intervals = true;
    }

    // could use read_files_parallel_inc_node_values to do fewer parallel read gymnastics
    // Tests originally showed read_files_extra_parallel_inc_node_values as faster in cloud run but slower on server; (adam 18th may)
    // Further tests show read_files_parallel_inc_node_values may be marginally faster in all cases (adam 19th May)
    let (node_values_2d, graph_walk, graph_routes) =
        read_files_extra_parallel_inc_node_values(input.year);

    let mut graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let mut graph_routes: TiVec<NodeID, NodeRoute> = TiVec::from(graph_routes);
    let mut node_values_2d: TiVec<NodeID, Vec<SubpurposeScore>> = TiVec::from(node_values_2d);

    let graph_walk_og_len = graph_walk.len();
    let graph_routes_og_len = graph_routes.len();
    let node_values_2d_og_len = node_values_2d.len();

    let time_of_day_ix = get_time_of_day_index(input.trip_start_seconds);

    // Make new routes nodes, and walking links from those nodes to new nodes
    for input_edges in input.graph_walk_additions.iter() {
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1]),
                cost: Cost(array[0]),
            });
        }
        graph_walk.push(NodeWalk {
            edges: edges,
            has_pt: true,
        });
    }

    // add timetables for new route nodes
    for timetable in input.graph_routes_additions.iter() {
        let mut edges: SmallVec<[EdgeRoute; 4]> = SmallVec::new();

        // POSSIBLE IMPROVEMENT: change next_stop_node as separate payload from python code. When this is done won't want to skip first edge as we do here
        // links to **!!&&$$ marker in connectivity/flask_app_PT/preproc_and_Functions_for_PT_app.py
        // For now (29th June 2023, Adam) need to skip as first row is empty
        for single_time in timetable.iter().skip(1) {
            edges.push(EdgeRoute {
                leavetime: SecondsPastMidnight(single_time[0]),
                cost: Cost(single_time[1]),
            });
        }

        // POSSIBLE IMPROVEMENT: change next_stop_node as separate payload from python code
        graph_routes.push(NodeRoute {
            next_stop_node: NodeID(timetable[0][0]),
            timetable: edges,
        });
    }

    // easiest way to remove route links from the graph: make has_PT False for specified node IDs
    // then floodfill_public_transport_purpose_scores() will act as if those routes aren't there
    let mut nodes_flipped_to_no_pt: Vec<NodeID> = vec![];
    for node in input.nodes_to_remove_routes_from.iter() {
        if graph_walk[*node].has_pt == true {
            nodes_flipped_to_no_pt.push(*node);
        }
        graph_walk[*node].has_pt = false;
    }

    // Adding walking connections from existing nodes to new route nodes
    for i in 0..input.graph_walk_updates_keys.len() {
        let node = input.graph_walk_updates_keys[i];

        // Optional improvement to make: Just modify in-place
        let mut edges: SmallVec<[EdgeWalk; 4]> = graph_walk[node].edges.clone();
        for array in &input.graph_walk_updates_additions[i] {
            edges.push(EdgeWalk {
                to: NodeID(array[1]),
                cost: Cost(array[0]),
            });
        }
        graph_walk[node].edges = edges;
    }

    // Add empty node values for new route nodes
    for _i in 0..input.graph_walk_additions.len() {
        let empty_vec: Vec<SubpurposeScore> = Vec::new();
        node_values_2d.push(empty_vec);
    }

    assert!(graph_routes.len() == graph_walk.len());
    assert!(node_values_2d.len() == graph_walk.len());

    // Add subpurpose values for new builds
    for new_build in &input.new_build_additions {
        let value_to_add = new_build[0] as f64;
        let index_of_nearest_node = new_build[1];
        let subpurpose_ix = new_build[2];

        // add node value to current score if one can be found for this node for the new build's subpurpose
        let mut loop_ix = 0;
        let mut found_existing_subpurpose = false;
        let values_vec_this_node = node_values_2d[NodeID(index_of_nearest_node)].to_vec();
        for subpurpose_score_pair in values_vec_this_node.iter() {
            if subpurpose_ix == subpurpose_score_pair.subpurpose_ix {
                node_values_2d[NodeID(index_of_nearest_node)][loop_ix].subpurpose_score +=
                    Score(value_to_add);
                found_existing_subpurpose = true;
            }
            loop_ix += 1;
        }

        // append to node_values_2d if no current value for that node's subpurpose
        if !found_existing_subpurpose {
            let subpurpose_value_to_add = SubpurposeScore {
                subpurpose_ix: subpurpose_ix,
                subpurpose_score: Score(value_to_add),
            };
            node_values_2d[NodeID(index_of_nearest_node)].push(subpurpose_value_to_add);
        }
    }

    let mut stop_rail_statuses_input = read_stop_rail_statuses(input.year);

    // takes about 8ms to extend with 20_000_000 false values
    // It had a lenght of all route nodes (just under 2m with July 2022 timetables)
    // We do this so that when we add new nodes, this isn't out of bounds
    // 20m + 2m = 22m, which is much more than the current number of nodes (~12m)
    let false_values = vec![false; 20_000_000];
    stop_rail_statuses_input.extend(false_values);

    let stop_rail_statuses: TiVec<NodeID, bool> = TiVec::from(stop_rail_statuses_input);

    let now = Instant::now();
    let indices = (0..input.start_nodes.len()).collect::<Vec<_>>();

    let results: Vec<FloodfillOutputOriginDestinationPair> = indices
        .par_iter()
        .map(|i| {
            floodfill_public_transport_purpose_scores(
                &graph_walk,
                &graph_routes,
                input.start_nodes[*i],
                input.trip_start_seconds,
                input.init_travel_times[*i],
                false,
                Cost(3600),
                &node_values_2d,
                &data.travel_time_relationships_all[time_of_day_ix],
                &input.target_destinations,
                &stop_rail_statuses,
                &data.small_medium_large_subpurpose_destinations,
                count_destinations_at_intervals,
                &input.original_time_intervals_to_store_destination_counts,
            )
        })
        .collect();
    println!("Getting destinations and scores took {:?}", now.elapsed());

    // *********
    // Reset graph_walk, graph_routes and node_values_2d
    // To drop effects of added or removed routes, and new builds
    // Not needed if we load the data in on every API call, but useful
    // if we store the data in AppState, which we may do for batch
    // processing of changes on a server, as part of the lowest-hanging-fruit-optimiser
    // *********

    // pop the nodes added
    let original_graph_walk_len = graph_walk.len() - input.graph_walk_additions.len();
    graph_walk.truncate(original_graph_walk_len);

    let original_graph_routes_len = graph_routes.len() - input.graph_walk_additions.len();
    graph_routes.truncate(original_graph_routes_len);

    // re-adding any routes which were removed
    for node_id in nodes_flipped_to_no_pt {
        graph_walk[node_id].has_pt = true;
    }

    // remove connections from existing nodes to new nodes - the same new nodes we deleted in the lines above
    for i in 0..input.graph_walk_updates_keys.len() {
        let node = input.graph_walk_updates_keys[i];

        // Possible improvement: modify in-place without clone()
        let mut edges: SmallVec<[EdgeWalk; 4]> = graph_walk[node].edges.clone();

        let original_edges_count = edges.len() - input.graph_walk_updates_additions[i].len();
        edges.truncate(original_edges_count);
        graph_walk[node].edges = edges;
    }

    // remove new nodes from node values
    let original_node_values_len = node_values_2d.len() - input.graph_walk_additions.len();
    node_values_2d.truncate(original_node_values_len);

    // remove subpurpose values from new builds
    for new_build in &input.new_build_additions {
        let value_to_add = new_build[0] as f64;
        let index_of_nearest_node = new_build[1];
        let subpurpose_ix = new_build[2];

        // subtract node value from current score if one can be found for this node for the new build's subpurpose
        let mut loop_ix = 0;
        let values_vec_this_node = node_values_2d[NodeID(index_of_nearest_node)].to_vec();
        for subpurpose_score_pair in values_vec_this_node.iter() {
            if subpurpose_ix == subpurpose_score_pair.subpurpose_ix {
                node_values_2d[NodeID(index_of_nearest_node)][loop_ix].subpurpose_score -=
                    Score(value_to_add);
            }
            loop_ix += 1;
        }
    }

    assert!(graph_walk_og_len == graph_walk.len());
    assert!(graph_routes_og_len == graph_routes.len());
    assert!(node_values_2d_og_len == node_values_2d.len());

    serde_json::to_string(&results).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

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
    let small_medium_large_subpurpose_destinations_input = read_small_medium_large_subpurpose_destinations("PT");
    let small_medium_large_subpurpose_destinations: TiVec<NodeID, Vec<SubpurposeSmallMediumLargeCount>> = TiVec::from(small_medium_large_subpurpose_destinations_input);
    
    let app_state = web::Data::new(AppState {
        travel_time_relationships_all,
        small_medium_large_subpurpose_destinations,
    });

    // The 500MB warning is wrong, the decorator on line below silences it
    #[allow(deprecated)]
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
